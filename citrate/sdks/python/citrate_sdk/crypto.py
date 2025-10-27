"""
Cryptographic utilities for Citrate SDK
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

from .errors import CitrateError
from .finite_field import split_secret_bytes, reconstruct_secret_bytes
from .ecdh_real import ECDHManager


class KeyManager:
    """
    Manages cryptographic keys for Citrate operations.

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
        private_key_bytes = None
        if private_key:
            # Use Ethereum private key for ECDH as well
            if private_key.startswith('0x'):
                private_key = private_key[2:]
            private_key_bytes = bytes.fromhex(private_key)

        self.ecdh_manager = ECDHManager(private_key_bytes)

    def get_address(self) -> str:
        """Get Ethereum address"""
        return self.account.address

    def get_private_key(self) -> str:
        """Get private key as hex string"""
        return self.account.key.hex()

    def get_public_key(self) -> str:
        """Get ECDH public key for sharing"""
        public_bytes = self.ecdh_manager.get_public_key_compressed()
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
            return "0x" + signed_txn.raw_transaction.hex()
        except Exception as e:
            raise CitrateError(f"Transaction signing failed: {str(e)}")

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
            raise CitrateError(f"Model decryption failed: {str(e)}")

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
            raise CitrateError(f"Data decryption failed: {str(e)}")

    def derive_shared_key(self, peer_public_key: str) -> bytes:
        """
        Derive shared key using ECDH.

        Args:
            peer_public_key: Hex-encoded peer public key

        Returns:
            32-byte shared key
        """
        try:
            peer_key_bytes = bytes.fromhex(peer_public_key)
            return self.ecdh_manager.derive_shared_secret(
                peer_key_bytes,
                salt=b"citrate-model-encryption",
                info=b"shared-key-derivation"
            )

        except Exception as e:
            raise CitrateError(f"Key derivation failed: {str(e)}")

    def _encrypt_key_for_owner(self, key: bytes) -> str:
        """Encrypt key for model owner using proper key wrapping"""
        # Use HKDF for proper key derivation from account key
        salt = secrets.token_bytes(32)
        hkdf = HKDF(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            info=b'citrate-key-wrapping',
            backend=default_backend()
        )
        owner_key = hkdf.derive(self.account.key)

        nonce = secrets.token_bytes(12)
        aesgcm = AESGCM(owner_key)
        encrypted_key = aesgcm.encrypt(nonce, key, None)

        return json.dumps({
            "encrypted_key": encrypted_key.hex(),
            "nonce": nonce.hex(),
            "salt": salt.hex()
        })

    def _decrypt_key_from_owner(self, encrypted_key_package: str) -> bytes:
        """Decrypt key for model owner using proper key derivation"""
        package = json.loads(encrypted_key_package)
        encrypted_key = bytes.fromhex(package["encrypted_key"])
        nonce = bytes.fromhex(package["nonce"])
        salt = bytes.fromhex(package["salt"])

        # Derive owner key using same HKDF parameters
        hkdf = HKDF(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            info=b'citrate-key-wrapping',
            backend=default_backend()
        )
        owner_key = hkdf.derive(self.account.key)

        aesgcm = AESGCM(owner_key)
        return aesgcm.decrypt(nonce, encrypted_key, None)

    def _create_key_shares(self, key: bytes, threshold: int, total: int) -> List[Dict[str, str]]:
        """Create Shamir's secret shares for key using proper finite field arithmetic"""
        shares_tuples = split_secret_bytes(key, threshold, total)

        shares = []
        for x, share_bytes in shares_tuples:
            shares.append({
                "x": str(x),
                "y": share_bytes.hex(),
                "threshold": str(threshold)
            })

        return shares

    def reconstruct_key_from_shares(self, shares: List[Dict[str, str]]) -> bytes:
        """Reconstruct key from Shamir's shares using proper Lagrange interpolation"""
        if not shares:
            raise CitrateError("No shares provided")

        threshold = int(shares[0]["threshold"])
        if len(shares) < threshold:
            raise CitrateError("Insufficient shares for key reconstruction")

        # Convert shares back to tuples format
        shares_tuples = []
        for share in shares:
            x = int(share["x"])
            y = bytes.fromhex(share["y"])
            shares_tuples.append((x, y))

        return reconstruct_secret_bytes(shares_tuples, threshold)


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