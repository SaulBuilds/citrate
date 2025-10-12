"""
Cryptographic utilities for Lattice SDK
"""

import hashlib
import secrets
import json
from typing import Tuple, Dict, Any, List
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.backends import default_backend
from eth_account import Account
from eth_account.signers.local import LocalAccount

from .errors import LatticeError


class KeyManager:
    """
    Manages cryptographic keys for Lattice operations.

    Handles:
    - Ethereum account management
    - Model encryption/decryption
    - Key derivation and sharing
    - Transaction signing
    """

    def __init__(self, private_key: str = None):
        """
        Initialize key manager.

        Args:
            private_key: Hex-encoded private key, or None to generate new key
        """
        if private_key:
            if private_key.startswith('0x'):
                private_key = private_key[2:]
            self.account: LocalAccount = Account.from_key(private_key)
        else:
            self.account: LocalAccount = Account.create()

        # Generate ECDH key pair for model encryption
        self.ecdh_private_key = ec.generate_private_key(ec.SECP256K1(), default_backend())
        self.ecdh_public_key = self.ecdh_private_key.public_key()

    def get_address(self) -> str:
        """Get Ethereum address"""
        return self.account.address

    def get_private_key(self) -> str:
        """Get private key as hex string"""
        return self.account.key.hex()

    def get_public_key(self) -> str:
        """Get ECDH public key for sharing"""
        public_bytes = self.ecdh_public_key.public_numbers().x.to_bytes(32, 'big')
        return public_bytes.hex()

    def sign_transaction(self, transaction: Dict[str, Any]) -> str:
        """
        Sign Ethereum transaction.

        Args:
            transaction: Transaction dict with standard fields

        Returns:
            Signed transaction as hex string
        """
        try:
            signed_txn = self.account.sign_transaction(transaction)
            return signed_txn.raw_transaction.hex()
        except Exception as e:
            raise LatticeError(f"Transaction signing failed: {str(e)}")

    def encrypt_model(
        self,
        model_data: bytes,
        config: 'EncryptionConfig'
    ) -> Tuple[bytes, Dict[str, Any]]:
        """
        Encrypt model data with AES-256-GCM.

        Args:
            model_data: Raw model file bytes
            config: Encryption configuration

        Returns:
            Tuple of (encrypted_data, encryption_metadata)
        """
        # Generate random key and nonce
        key = secrets.token_bytes(32)  # 256-bit key
        nonce = secrets.token_bytes(12)  # 96-bit nonce for GCM

        # Encrypt data
        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(nonce, model_data, None)

        # Create encryption metadata
        metadata = {
            "algorithm": config.algorithm,
            "nonce": nonce.hex(),
            "key_derivation": config.key_derivation,
            "encrypted_key": self._encrypt_key_for_owner(key),
            "access_control": config.access_control
        }

        # Add threshold sharing if enabled
        if config.threshold_shares > 0:
            key_shares = self._create_key_shares(key, config.threshold_shares, config.total_shares)
            metadata["key_shares"] = key_shares

        return ciphertext, metadata

    def decrypt_model(
        self,
        encrypted_data: bytes,
        metadata: Dict[str, Any]
    ) -> bytes:
        """
        Decrypt model data.

        Args:
            encrypted_data: Encrypted model bytes
            metadata: Encryption metadata from deployment

        Returns:
            Decrypted model data
        """
        try:
            # Extract encryption parameters
            nonce = bytes.fromhex(metadata["nonce"])
            encrypted_key = metadata["encrypted_key"]

            # Decrypt the encryption key
            key = self._decrypt_key_from_owner(encrypted_key)

            # Decrypt model data
            aesgcm = AESGCM(key)
            plaintext = aesgcm.decrypt(nonce, encrypted_data, None)

            return plaintext

        except Exception as e:
            raise LatticeError(f"Model decryption failed: {str(e)}")

    def encrypt_data(self, data: str) -> str:
        """Encrypt arbitrary string data"""
        data_bytes = data.encode('utf-8')
        key = secrets.token_bytes(32)
        nonce = secrets.token_bytes(12)

        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(nonce, data_bytes, None)

        # Package with key and nonce
        package = {
            "ciphertext": ciphertext.hex(),
            "nonce": nonce.hex(),
            "key": key.hex()
        }

        return json.dumps(package)

    def decrypt_data(self, encrypted_package: str) -> str:
        """Decrypt string data from encrypt_data"""
        try:
            package = json.loads(encrypted_package)
            ciphertext = bytes.fromhex(package["ciphertext"])
            nonce = bytes.fromhex(package["nonce"])
            key = bytes.fromhex(package["key"])

            aesgcm = AESGCM(key)
            plaintext = aesgcm.decrypt(nonce, ciphertext, None)

            return plaintext.decode('utf-8')

        except Exception as e:
            raise LatticeError(f"Data decryption failed: {str(e)}")

    def derive_shared_key(self, peer_public_key: str) -> bytes:
        """
        Derive shared key using ECDH.

        Args:
            peer_public_key: Hex-encoded peer public key

        Returns:
            32-byte shared key
        """
        try:
            # Reconstruct peer public key
            peer_key_bytes = bytes.fromhex(peer_public_key)
            peer_public_numbers = ec.EllipticCurvePublicNumbers(
                x=int.from_bytes(peer_key_bytes, 'big'),
                y=0,  # Compressed format
                curve=ec.SECP256K1()
            )
            peer_key = peer_public_numbers.public_key(default_backend())

            # Perform ECDH
            shared_key = self.ecdh_private_key.exchange(ec.ECDH(), peer_key)

            # Derive final key using HKDF
            hkdf = HKDF(
                algorithm=hashes.SHA256(),
                length=32,
                salt=b"lattice-model-encryption",
                info=b"shared-key-derivation",
                backend=default_backend()
            )

            return hkdf.derive(shared_key)

        except Exception as e:
            raise LatticeError(f"Key derivation failed: {str(e)}")

    def _encrypt_key_for_owner(self, key: bytes) -> str:
        """Encrypt key for model owner"""
        # Simple encryption with owner's key
        # In production, use proper key wrapping
        owner_key = hashlib.sha256(self.account.key).digest()
        nonce = secrets.token_bytes(12)

        aesgcm = AESGCM(owner_key)
        encrypted_key = aesgcm.encrypt(nonce, key, None)

        return json.dumps({
            "encrypted_key": encrypted_key.hex(),
            "nonce": nonce.hex()
        })

    def _decrypt_key_from_owner(self, encrypted_key_package: str) -> bytes:
        """Decrypt key for model owner"""
        package = json.loads(encrypted_key_package)
        encrypted_key = bytes.fromhex(package["encrypted_key"])
        nonce = bytes.fromhex(package["nonce"])

        owner_key = hashlib.sha256(self.account.key).digest()
        aesgcm = AESGCM(owner_key)

        return aesgcm.decrypt(nonce, encrypted_key, None)

    def _create_key_shares(self, key: bytes, threshold: int, total: int) -> List[Dict[str, str]]:
        """Create Shamir's secret shares for key"""
        # Simplified implementation - in production use proper finite field arithmetic
        shares = []
        for i in range(1, total + 1):
            # Generate polynomial evaluation at point i
            share_value = hashlib.sha256(key + i.to_bytes(4, 'big')).digest()
            shares.append({
                "x": str(i),
                "y": share_value.hex(),
                "threshold": str(threshold)
            })

        return shares

    def reconstruct_key_from_shares(self, shares: List[Dict[str, str]]) -> bytes:
        """Reconstruct key from Shamir's shares"""
        # Simplified implementation
        # In production, use proper Lagrange interpolation
        if len(shares) < int(shares[0]["threshold"]):
            raise LatticeError("Insufficient shares for key reconstruction")

        # For demo, just return hash of combined shares
        combined = b""
        for share in shares[:int(shares[0]["threshold"])]:
            combined += bytes.fromhex(share["y"])

        return hashlib.sha256(combined).digest()


class EncryptionConfig:
    """Configuration for model encryption"""

    def __init__(
        self,
        algorithm: str = "AES-256-GCM",
        key_derivation: str = "HKDF-SHA256",
        access_control: bool = True,
        threshold_shares: int = 0,
        total_shares: int = 0
    ):
        self.algorithm = algorithm
        self.key_derivation = key_derivation
        self.access_control = access_control
        self.threshold_shares = threshold_shares
        self.total_shares = total_shares


def generate_model_key() -> str:
    """Generate random 256-bit key for model encryption"""
    return secrets.token_bytes(32).hex()


def hash_model_data(data: bytes) -> str:
    """Generate SHA-256 hash of model data"""
    return hashlib.sha256(data).hexdigest()


def verify_model_integrity(data: bytes, expected_hash: str) -> bool:
    """Verify model data integrity against expected hash"""
    actual_hash = hash_model_data(data)
    return actual_hash == expected_hash