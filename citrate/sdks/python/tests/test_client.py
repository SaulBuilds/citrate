"""
Unit tests for Citrate SDK client
"""

import pytest
import json
import tempfile
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

from citrate_sdk import CitrateClient, ModelConfig, ModelType, AccessType
from citrate_sdk.errors import CitrateError, ModelNotFoundError
from citrate_sdk.crypto import KeyManager


class TestCitrateClient:
    """Test cases for CitrateClient"""

    def setup_method(self):
        """Setup test fixtures"""
        self.mock_private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        self.mock_rpc_url = "http://localhost:8545"

    @patch('requests.Session.post')
    def test_rpc_call_success(self, mock_post):
        """Test successful RPC call"""
        # Setup mock response
        mock_response = Mock()
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "result": "0x1",
            "id": 1
        }
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        client = CitrateClient(self.mock_rpc_url)
        result = client._rpc_call("eth_chainId")

        assert result == "0x1"
        mock_post.assert_called_once()

    @patch('requests.Session.post')
    def test_rpc_call_error(self, mock_post):
        """Test RPC call with error response"""
        mock_response = Mock()
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "error": {"code": -32602, "message": "Invalid params"},
            "id": 1
        }
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        client = CitrateClient(self.mock_rpc_url)

        with pytest.raises(CitrateError, match="RPC error: Invalid params"):
            client._rpc_call("invalid_method")

    @patch('requests.Session.post')
    def test_get_chain_id(self, mock_post):
        """Test chain ID retrieval"""
        mock_response = Mock()
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "result": "0x539",  # 1337 in hex
            "id": 1
        }
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        client = CitrateClient(self.mock_rpc_url)
        chain_id = client.get_chain_id()

        assert chain_id == "0x539"

    @patch('requests.Session.post')
    def test_get_balance(self, mock_post):
        """Test balance retrieval"""
        mock_response = Mock()
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "result": "0xde0b6b3a7640000",  # 1 ETH in wei
            "id": 1
        }
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        client = CitrateClient(self.mock_rpc_url)
        balance = client.get_balance("0x1234567890123456789012345678901234567890")

        assert balance == 1000000000000000000  # 1 ETH in wei

    def test_client_initialization_with_key(self):
        """Test client initialization with private key"""
        client = CitrateClient(self.mock_rpc_url, self.mock_private_key)

        assert client.rpc_url == self.mock_rpc_url
        assert client.key_manager is not None
        assert client.key_manager.get_private_key() == self.mock_private_key[2:]  # Without 0x prefix

    def test_client_initialization_without_key(self):
        """Test client initialization without private key"""
        client = CitrateClient(self.mock_rpc_url)

        assert client.rpc_url == self.mock_rpc_url
        assert client.key_manager is None

    @patch.object(CitrateClient, '_rpc_call')
    @patch.object(CitrateClient, '_upload_to_ipfs')
    @patch.object(CitrateClient, '_send_transaction')
    @patch.object(CitrateClient, '_wait_for_receipt')
    @patch.object(CitrateClient, '_extract_model_id_from_receipt')
    def test_deploy_model(self, mock_extract, mock_wait, mock_send, mock_upload, mock_rpc):
        """Test model deployment"""
        # Setup mocks
        mock_upload.return_value = "QmTestHash123"
        mock_send.return_value = "0xTransactionHash"
        mock_wait.return_value = {"status": "0x1", "logs": []}
        mock_extract.return_value = "model_123"

        # Create temporary model file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump({"test": "model"}, f)
            model_path = f.name

        try:
            client = CitrateClient(self.mock_rpc_url, self.mock_private_key)
            config = ModelConfig(
                name="Test Model",
                model_type=ModelType.CUSTOM,
                access_type=AccessType.PUBLIC
            )

            deployment = client.deploy_model(model_path, config)

            assert deployment.model_id == "model_123"
            assert deployment.tx_hash == "0xTransactionHash"
            assert deployment.ipfs_hash == "QmTestHash123"
            assert deployment.encrypted == False

        finally:
            Path(model_path).unlink()

    def test_deploy_model_without_key(self):
        """Test model deployment without private key raises error"""
        client = CitrateClient(self.mock_rpc_url)  # No private key
        config = ModelConfig()

        with pytest.raises(CitrateError, match="Private key required"):
            client.deploy_model("nonexistent.json", config)

    def test_deploy_model_nonexistent_file(self):
        """Test model deployment with nonexistent file"""
        client = CitrateClient(self.mock_rpc_url, self.mock_private_key)
        config = ModelConfig()

        with pytest.raises(CitrateError, match="Model file not found"):
            client.deploy_model("nonexistent.json", config)

    @patch.object(CitrateClient, '_rpc_call')
    @patch.object(CitrateClient, '_send_transaction')
    @patch.object(CitrateClient, '_wait_for_receipt')
    @patch.object(CitrateClient, '_extract_inference_output')
    def test_inference(self, mock_extract, mock_wait, mock_send, mock_rpc):
        """Test inference execution"""
        # Setup mocks
        mock_send.return_value = "0xInferenceHash"
        mock_wait.return_value = {"status": "0x1", "gasUsed": "0x1234"}
        mock_extract.return_value = {"prediction": "cat", "confidence": 0.95}

        client = CitrateClient(self.mock_rpc_url, self.mock_private_key)

        result = client.inference(
            model_id="model_123",
            input_data={"image": "test_image"}
        )

        assert result.model_id == "model_123"
        assert result.output_data["prediction"] == "cat"
        assert result.gas_used == int("0x1234", 16)

    @patch.object(CitrateClient, '_rpc_call')
    def test_get_model_info_success(self, mock_rpc):
        """Test successful model info retrieval"""
        mock_rpc.return_value = {
            "model_id": "model_123",
            "name": "Test Model",
            "owner": "0x1234567890123456789012345678901234567890"
        }

        client = CitrateClient(self.mock_rpc_url)
        info = client.get_model_info("model_123")

        assert info["model_id"] == "model_123"
        assert info["name"] == "Test Model"

    @patch.object(CitrateClient, '_rpc_call')
    def test_get_model_info_not_found(self, mock_rpc):
        """Test model info retrieval for nonexistent model"""
        mock_rpc.return_value = None

        client = CitrateClient(self.mock_rpc_url)

        with pytest.raises(ModelNotFoundError, match="Model not found: invalid_model"):
            client.get_model_info("invalid_model")

    @patch.object(CitrateClient, '_rpc_call')
    def test_list_models(self, mock_rpc):
        """Test model listing"""
        mock_rpc.return_value = [
            {"model_id": "model_1", "name": "Model 1"},
            {"model_id": "model_2", "name": "Model 2"}
        ]

        client = CitrateClient(self.mock_rpc_url)
        models = client.list_models(limit=2)

        assert len(models) == 2
        assert models[0]["model_id"] == "model_1"
        assert models[1]["model_id"] == "model_2"


class TestModelConfig:
    """Test cases for ModelConfig"""

    def test_default_config(self):
        """Test default model configuration"""
        config = ModelConfig()

        assert config.name == ""
        assert config.model_type == ModelType.COREML
        assert config.access_type == AccessType.PUBLIC
        assert config.encrypted == False
        assert config.access_price == 0

    def test_custom_config(self):
        """Test custom model configuration"""
        config = ModelConfig(
            name="Custom Model",
            description="A custom AI model",
            model_type=ModelType.PYTORCH,
            access_type=AccessType.PAID,
            access_price=1000000000000000000,  # 1 ETH
            encrypted=True,
            tags=["ai", "custom"]
        )

        assert config.name == "Custom Model"
        assert config.model_type == ModelType.PYTORCH
        assert config.access_type == AccessType.PAID
        assert config.access_price == 1000000000000000000
        assert config.encrypted == True
        assert "ai" in config.tags


class TestErrorHandling:
    """Test error handling scenarios"""

    @patch('requests.Session.post')
    def test_network_error(self, mock_post):
        """Test network error handling"""
        mock_post.side_effect = Exception("Network error")

        client = CitrateClient("http://localhost:8545")

        with pytest.raises(CitrateError, match="Network error"):
            client._rpc_call("test_method")

    @patch('requests.Session.post')
    def test_invalid_json_response(self, mock_post):
        """Test invalid JSON response handling"""
        mock_response = Mock()
        mock_response.json.side_effect = json.JSONDecodeError("Invalid JSON", "", 0)
        mock_response.raise_for_status.return_value = None
        mock_post.return_value = mock_response

        client = CitrateClient("http://localhost:8545")

        with pytest.raises(CitrateError, match="Invalid JSON response"):
            client._rpc_call("test_method")