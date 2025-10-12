"""
Lattice Client - Main SDK interface for Lattice blockchain interaction
"""

import json
import requests
from typing import Dict, Any, List, Optional, Union
from dataclasses import dataclass, asdict
from pathlib import Path
import hashlib
import time

from .models import ModelConfig, ModelDeployment, InferenceRequest, InferenceResult
from .crypto import EncryptionConfig, KeyManager
from .errors import LatticeError, ModelNotFoundError, InsufficientFundsError
from .ipfs import upload_to_ipfs


class LatticeClient:
    """
    Main client for interacting with Lattice blockchain.

    Provides methods for:
    - Model deployment and management
    - Inference execution
    - Encryption and access control
    - Payment and revenue sharing
    """

    def __init__(self, rpc_url: str = "http://localhost:8545", private_key: Optional[str] = None):
        """
        Initialize Lattice client.

        Args:
            rpc_url: RPC endpoint URL
            private_key: Optional private key for transactions
        """
        self.rpc_url = rpc_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'User-Agent': 'lattice-python-sdk/0.1.0'
        })

        self.key_manager = KeyManager(private_key) if private_key else None
        self._request_id = 0

    def _next_request_id(self) -> int:
        """Get next JSON-RPC request ID"""
        self._request_id += 1
        return self._request_id

    def _rpc_call(self, method: str, params: List[Any] = None) -> Any:
        """
        Make JSON-RPC call to Lattice node.

        Args:
            method: RPC method name
            params: Method parameters

        Returns:
            RPC response result

        Raises:
            LatticeError: If RPC call fails
        """
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or [],
            "id": self._next_request_id()
        }

        try:
            response = self.session.post(self.rpc_url, json=payload, timeout=30)
            response.raise_for_status()

            data = response.json()
            if "error" in data:
                raise LatticeError(f"RPC error: {data['error']['message']}")

            return data.get("result")

        except requests.exceptions.RequestException as e:
            raise LatticeError(f"Network error: {str(e)}")
        except json.JSONDecodeError as e:
            raise LatticeError(f"Invalid JSON response: {str(e)}")

    def get_chain_id(self) -> int:
        """Get blockchain chain ID"""
        return self._rpc_call("eth_chainId")

    def get_balance(self, address: str) -> int:
        """Get account balance in wei"""
        result = self._rpc_call("eth_getBalance", [address, "latest"])
        return int(result, 16)

    def get_nonce(self, address: str) -> int:
        """Get account transaction nonce"""
        result = self._rpc_call("eth_getTransactionCount", [address, "pending"])
        return int(result, 16)

    def deploy_model(
        self,
        model_path: Union[str, Path],
        config: ModelConfig
    ) -> ModelDeployment:
        """
        Deploy an AI model to Lattice blockchain.

        Args:
            model_path: Path to model file (.mlpackage, .onnx, etc.)
            config: Model configuration including encryption and access settings

        Returns:
            ModelDeployment with deployment details

        Raises:
            LatticeError: If deployment fails
        """
        if not self.key_manager:
            raise LatticeError("Private key required for model deployment")

        model_path = Path(model_path)
        if not model_path.exists():
            raise LatticeError(f"Model file not found: {model_path}")

        # Read and hash model file
        model_data = model_path.read_bytes()
        model_hash = hashlib.sha256(model_data).hexdigest()

        # Encrypt model if requested
        encrypted_data = None
        encryption_metadata = None

        if config.encrypted:
            if not config.encryption_config:
                config.encryption_config = EncryptionConfig()

            encrypted_data, encryption_metadata = self.key_manager.encrypt_model(
                model_data, config.encryption_config
            )

        # Upload to IPFS
        ipfs_hash = self._upload_to_ipfs(encrypted_data or model_data)

        # Deploy to blockchain
        tx_data = {
            "model_hash": model_hash,
            "ipfs_hash": ipfs_hash,
            "encrypted": config.encrypted,
            "access_price": config.access_price,
            "access_list": config.access_list or [],
            "metadata": config.metadata or {}
        }

        if encryption_metadata:
            tx_data["encryption_metadata"] = encryption_metadata

        # Call model deployment precompile (0x0100)
        tx_hash = self._send_transaction("0x0100000000000000000000000000000000000100", tx_data)

        # Wait for confirmation
        receipt = self._wait_for_receipt(tx_hash)

        # Extract model ID from logs
        model_id = self._extract_model_id_from_receipt(receipt)

        return ModelDeployment(
            model_id=model_id,
            tx_hash=tx_hash,
            ipfs_hash=ipfs_hash,
            encrypted=config.encrypted,
            access_price=config.access_price,
            deployment_time=int(time.time())
        )

    def inference(
        self,
        model_id: str,
        input_data: Dict[str, Any],
        encrypted: bool = False,
        max_gas: int = 1000000
    ) -> InferenceResult:
        """
        Execute inference on deployed model.

        Args:
            model_id: Deployed model identifier
            input_data: Input data for inference
            encrypted: Whether to use encrypted inference
            max_gas: Maximum gas limit for execution

        Returns:
            InferenceResult with outputs and metadata

        Raises:
            ModelNotFoundError: If model doesn't exist
            InsufficientFundsError: If insufficient funds for inference
            LatticeError: For other execution errors
        """
        # Prepare inference request
        request = InferenceRequest(
            model_id=model_id,
            input_data=input_data,
            encrypted=encrypted,
            timestamp=int(time.time())
        )

        # Encrypt input if needed
        if encrypted and self.key_manager:
            encrypted_input = self.key_manager.encrypt_data(json.dumps(input_data))
            request.input_data = {"encrypted": encrypted_input}

        # Call inference precompile (0x0101)
        tx_data = asdict(request)
        tx_hash = self._send_transaction(
            "0x0100000000000000000000000000000000000101",
            tx_data,
            gas_limit=max_gas
        )

        # Wait for execution
        receipt = self._wait_for_receipt(tx_hash)

        # Extract results from logs
        output_data = self._extract_inference_output(receipt)

        # Decrypt output if encrypted
        if encrypted and self.key_manager and "encrypted" in output_data:
            decrypted_output = self.key_manager.decrypt_data(output_data["encrypted"])
            output_data = json.loads(decrypted_output)

        return InferenceResult(
            model_id=model_id,
            output_data=output_data,
            gas_used=receipt.get("gasUsed", 0),
            execution_time=receipt.get("executionTime", 0),
            tx_hash=tx_hash
        )

    def get_model_info(self, model_id: str) -> Dict[str, Any]:
        """Get model deployment information"""
        params = [model_id]
        result = self._rpc_call("lattice_getModelInfo", params)

        if not result:
            raise ModelNotFoundError(f"Model not found: {model_id}")

        return result

    def list_models(self, owner: Optional[str] = None, limit: int = 100) -> List[Dict[str, Any]]:
        """List deployed models"""
        params = [owner, limit] if owner else [limit]
        return self._rpc_call("lattice_listModels", params)

    def purchase_model_access(self, model_id: str, payment_amount: int) -> str:
        """Purchase access to a paid model"""
        if not self.key_manager:
            raise LatticeError("Private key required for purchases")

        tx_data = {
            "model_id": model_id,
            "payment_amount": payment_amount
        }

        return self._send_transaction(
            "0x0100000000000000000000000000000000000104",  # Access control precompile
            tx_data,
            value=payment_amount
        )

    def _upload_to_ipfs(self, data: bytes) -> str:
        """Upload data to IPFS and return hash"""
        try:
            return upload_to_ipfs(data)
        except Exception as e:
            # Fallback to local hash if IPFS is unavailable
            print(f"Warning: IPFS upload failed ({e}), using local hash fallback")
            return f"fallback_{hashlib.sha256(data).hexdigest()}"

    def _send_transaction(
        self,
        to_address: str,
        data: Dict[str, Any],
        value: int = 0,
        gas_limit: int = 500000
    ) -> str:
        """Send transaction to blockchain"""
        if not self.key_manager:
            raise LatticeError("Private key required for transactions")

        # Get account info
        from_address = self.key_manager.get_address()
        nonce = self.get_nonce(from_address)

        # Build transaction
        tx = {
            "from": from_address,
            "to": to_address,
            "value": hex(value),
            "gas": hex(gas_limit),
            "gasPrice": hex(20_000_000_000),  # 20 gwei
            "nonce": hex(nonce),
            "data": "0x" + json.dumps(data).encode().hex()
        }

        # Sign transaction
        signed_tx = self.key_manager.sign_transaction(tx)

        # Send raw transaction
        return self._rpc_call("eth_sendRawTransaction", [signed_tx])

    def _wait_for_receipt(self, tx_hash: str, timeout: int = 60) -> Dict[str, Any]:
        """Wait for transaction receipt"""
        start_time = time.time()

        while time.time() - start_time < timeout:
            try:
                receipt = self._rpc_call("eth_getTransactionReceipt", [tx_hash])
                if receipt:
                    return receipt
            except LatticeError:
                pass

            time.sleep(1)

        raise LatticeError(f"Transaction timeout: {tx_hash}")

    def _extract_model_id_from_receipt(self, receipt: Dict[str, Any]) -> str:
        """Extract model ID from deployment receipt logs"""
        logs = receipt.get("logs", [])
        for log in logs:
            # Look for ModelDeployed event
            if log.get("topics", []):
                topic = log["topics"][0]
                if topic.startswith("0x" + "ModelDeployed".encode().hex()[:16]):
                    # Extract model ID from log data
                    return log["data"][:66]  # First 32 bytes as hex

        raise LatticeError("Model ID not found in deployment receipt")

    def _extract_inference_output(self, receipt: Dict[str, Any]) -> Dict[str, Any]:
        """Extract inference output from execution receipt"""
        logs = receipt.get("logs", [])
        for log in logs:
            if log.get("topics", []):
                topic = log["topics"][0]
                if topic.startswith("0x" + "InferenceComplete".encode().hex()[:16]):
                    # Decode output data from log
                    data_hex = log["data"]
                    data_bytes = bytes.fromhex(data_hex[2:])
                    return json.loads(data_bytes.decode())

        raise LatticeError("Inference output not found in receipt")