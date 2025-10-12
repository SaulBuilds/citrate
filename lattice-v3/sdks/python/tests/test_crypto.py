"""
Unit tests for Lattice SDK crypto module
"""

import pytest
import json
from unittest.mock import patch, Mock

from lattice_sdk.crypto import (
    KeyManager, EncryptionConfig, generate_model_key,
    hash_model_data, verify_model_integrity
)
from lattice_sdk.errors import LatticeError


class TestKeyManager:
    """Test cases for KeyManager"""

    def test_initialization_with_private_key(self):
        """Test KeyManager initialization with existing private key"""
        private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        key_manager = KeyManager(private_key)

        assert key_manager.get_private_key() == private_key
        assert key_manager.get_address().startswith("0x")
        assert len(key_manager.get_address()) == 42

    def test_initialization_with_0x_prefix(self):
        """Test KeyManager initialization with 0x prefix"""
        private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        key_manager = KeyManager(private_key)

        # Should strip 0x prefix
        assert key_manager.get_private_key() == private_key[2:]

    def test_initialization_without_key(self):
        """Test KeyManager initialization with generated key"""
        key_manager = KeyManager()

        private_key = key_manager.get_private_key()
        address = key_manager.get_address()

        assert len(private_key) == 64  # 32 bytes in hex
        assert address.startswith("0x")
        assert len(address) == 42

    def test_get_public_key(self):
        """Test public key generation"""
        key_manager = KeyManager()
        public_key = key_manager.get_public_key()

        assert len(public_key) == 64  # 32 bytes in hex
        assert all(c in "0123456789abcdef" for c in public_key)

    def test_encrypt_decrypt_data_roundtrip(self):
        """Test data encryption/decryption roundtrip"""
        key_manager = KeyManager()
        original_data = "This is sensitive information"

        encrypted_data = key_manager.encrypt_data(original_data)
        decrypted_data = key_manager.decrypt_data(encrypted_data)

        assert decrypted_data == original_data
        assert encrypted_data != original_data

    def test_encrypt_decrypt_unicode_data(self):
        """Test encryption/decryption with unicode characters"""
        key_manager = KeyManager()
        original_data = "Test with émojis 🔒 and ünïcödé"

        encrypted_data = key_manager.encrypt_data(original_data)
        decrypted_data = key_manager.decrypt_data(encrypted_data)

        assert decrypted_data == original_data

    def test_encrypt_model_data(self):
        """Test model data encryption"""
        key_manager = KeyManager()
        model_data = b"Mock model weights and parameters"
        config = EncryptionConfig()

        encrypted_data, metadata = key_manager.encrypt_model(model_data, config)

        assert len(encrypted_data) > 0
        assert isinstance(metadata, dict)
        assert "algorithm" in metadata
        assert "nonce" in metadata
        assert "encrypted_key" in metadata

    def test_decrypt_model_data(self):
        """Test model data decryption"""
        key_manager = KeyManager()
        model_data = b"Mock model weights and parameters"
        config = EncryptionConfig()

        encrypted_data, metadata = key_manager.encrypt_model(model_data, config)
        decrypted_data = key_manager.decrypt_model(encrypted_data, metadata)

        assert decrypted_data == model_data

    def test_derive_shared_key(self):
        """Test ECDH shared key derivation"""
        alice = KeyManager()
        bob = KeyManager()

        alice_pubkey = alice.get_public_key()
        bob_pubkey = bob.get_public_key()

        # Both parties derive same shared key
        alice_shared = alice.derive_shared_key(bob_pubkey)
        bob_shared = bob.derive_shared_key(alice_pubkey)

        assert len(alice_shared) == 32
        assert len(bob_shared) == 32
        # With real ECDH implementation, shared keys should be identical
        assert alice_shared == bob_shared

    def test_key_shares_creation(self):
        """Test Shamir's secret sharing key creation"""
        key_manager = KeyManager()
        key = b"test_key_32_bytes_long_for_sharing"

        shares = key_manager._create_key_shares(key, threshold=2, total=3)

        assert len(shares) == 3
        for share in shares:
            assert "x" in share
            assert "y" in share
            assert "threshold" in share
            assert share["threshold"] == "2"

    def test_key_reconstruction(self):
        """Test key reconstruction from shares"""
        key_manager = KeyManager()
        original_key = b"test_key_32_bytes_long_for_sharing"

        shares = key_manager._create_key_shares(original_key, threshold=2, total=3)
        reconstructed_key = key_manager.reconstruct_key_from_shares(shares[:2])

        # With real Shamir's Secret Sharing, original key should be perfectly reconstructed
        assert reconstructed_key == original_key

    def test_insufficient_shares_error(self):
        """Test error when insufficient shares provided"""
        key_manager = KeyManager()
        shares = [
            {"x": "1", "y": "abc123", "threshold": "3"}
        ]

        with pytest.raises(LatticeError, match="Insufficient shares"):
            key_manager.reconstruct_key_from_shares(shares)

    def test_sign_transaction(self):
        """Test transaction signing"""
        key_manager = KeyManager()

        transaction = {
            "to": "0x1234567890123456789012345678901234567890",
            "value": "0x0",
            "gas": "0x5208",
            "gasPrice": "0x4a817c800",
            "nonce": "0x0",
            "data": "0x"
        }

        signed_tx = key_manager.sign_transaction(transaction)

        assert signed_tx.startswith("0x")
        assert len(signed_tx) > 100  # Signed transaction should be substantial length


class TestEncryptionConfig:
    """Test cases for EncryptionConfig"""

    def test_default_config(self):
        """Test default encryption configuration"""
        config = EncryptionConfig()

        assert config.algorithm == "AES-256-GCM"
        assert config.key_derivation == "HKDF-SHA256"
        assert config.access_control == True
        assert config.threshold_shares == 0
        assert config.total_shares == 0

    def test_custom_config(self):
        """Test custom encryption configuration"""
        config = EncryptionConfig(
            algorithm="AES-128-GCM",
            threshold_shares=3,
            total_shares=5,
            access_control=False
        )

        assert config.algorithm == "AES-128-GCM"
        assert config.threshold_shares == 3
        assert config.total_shares == 5
        assert config.access_control == False


class TestCryptoUtilities:
    """Test cases for crypto utility functions"""

    def test_generate_model_key(self):
        """Test model key generation"""
        key1 = generate_model_key()
        key2 = generate_model_key()

        assert len(key1) == 64  # 32 bytes in hex
        assert len(key2) == 64
        assert key1 != key2  # Should be random
        assert all(c in "0123456789abcdef" for c in key1)

    def test_hash_model_data(self):
        """Test model data hashing"""
        data1 = b"Model data version 1"
        data2 = b"Model data version 2"

        hash1 = hash_model_data(data1)
        hash2 = hash_model_data(data2)

        assert len(hash1) == 64  # SHA-256 hash in hex
        assert len(hash2) == 64
        assert hash1 != hash2  # Different data should produce different hashes

        # Same data should produce same hash
        hash1_repeat = hash_model_data(data1)
        assert hash1 == hash1_repeat

    def test_verify_model_integrity_valid(self):
        """Test model integrity verification with valid hash"""
        data = b"Test model data for integrity check"
        expected_hash = hash_model_data(data)

        is_valid = verify_model_integrity(data, expected_hash)
        assert is_valid == True

    def test_verify_model_integrity_invalid(self):
        """Test model integrity verification with invalid hash"""
        data = b"Test model data for integrity check"
        wrong_hash = "1234567890abcdef" * 4  # Wrong hash

        is_valid = verify_model_integrity(data, wrong_hash)
        assert is_valid == False

    def test_verify_model_integrity_tampered_data(self):
        """Test model integrity verification with tampered data"""
        original_data = b"Original model data"
        tampered_data = b"Tampered model data"
        original_hash = hash_model_data(original_data)

        is_valid = verify_model_integrity(tampered_data, original_hash)
        assert is_valid == False


class TestErrorScenarios:
    """Test error handling in crypto operations"""

    def test_decrypt_invalid_package(self):
        """Test decryption with invalid encrypted package"""
        key_manager = KeyManager()
        invalid_package = "invalid json data"

        with pytest.raises(LatticeError, match="Data decryption failed"):
            key_manager.decrypt_data(invalid_package)

    def test_decrypt_malformed_json(self):
        """Test decryption with malformed JSON package"""
        key_manager = KeyManager()
        malformed_package = '{"ciphertext": "invalid_hex"}'

        with pytest.raises(LatticeError, match="Data decryption failed"):
            key_manager.decrypt_data(malformed_package)

    def test_model_decryption_missing_metadata(self):
        """Test model decryption with missing metadata"""
        key_manager = KeyManager()
        encrypted_data = b"encrypted model data"
        incomplete_metadata = {"algorithm": "AES-256-GCM"}  # Missing nonce, encrypted_key

        with pytest.raises(LatticeError, match="Model decryption failed"):
            key_manager.decrypt_model(encrypted_data, incomplete_metadata)

    def test_derive_shared_key_invalid_pubkey(self):
        """Test shared key derivation with invalid public key"""
        key_manager = KeyManager()
        invalid_pubkey = "invalid_public_key_format"

        with pytest.raises(LatticeError, match="Key derivation failed"):
            key_manager.derive_shared_key(invalid_pubkey)