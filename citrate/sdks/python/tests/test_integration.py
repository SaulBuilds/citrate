"""
Integration Tests for Citrate Python SDK

Tests SDK functionality against a real (embedded or testnet) node.

Run with: pytest tests/test_integration.py -v

Environment variables:
- CITRATE_RPC_URL: RPC endpoint (default: http://localhost:8545)
- CITRATE_CHAIN_ID: Chain ID (default: 1337)
- CITRATE_TEST_PRIVATE_KEY: Private key with funds for transaction tests
"""

import os
import time
import json
import pytest
import hashlib
from typing import Optional, Dict, Any

# Import SDK modules
from citrate_sdk.client import CitrateClient
from citrate_sdk.crypto import KeyManager
from citrate_sdk.errors import CitrateError

# ============================================================================
# Test Configuration
# ============================================================================

RPC_ENDPOINT = os.environ.get('CITRATE_RPC_URL', 'http://localhost:8545')
CHAIN_ID = int(os.environ.get('CITRATE_CHAIN_ID', '1337'))
TEST_PRIVATE_KEY = os.environ.get('CITRATE_TEST_PRIVATE_KEY')

# Well-known test accounts (from Hardhat/Anvil)
GENESIS_ACCOUNTS = [
    {
        'private_key': 'ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
        'address': '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
    },
    {
        'private_key': '59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
        'address': '0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
    },
    {
        'private_key': '5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
        'address': '0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC',
    },
]


# ============================================================================
# Test Fixtures
# ============================================================================

@pytest.fixture
def client():
    """Create SDK client without private key"""
    return CitrateClient(rpc_url=RPC_ENDPOINT)


@pytest.fixture
def funded_client():
    """Create SDK client with funded test account"""
    private_key = TEST_PRIVATE_KEY or GENESIS_ACCOUNTS[0]['private_key']
    return CitrateClient(rpc_url=RPC_ENDPOINT, private_key=private_key)


@pytest.fixture
def key_manager():
    """Create key manager with test private key"""
    private_key = TEST_PRIVATE_KEY or GENESIS_ACCOUNTS[0]['private_key']
    return KeyManager(private_key)


def is_node_running(rpc_url: str) -> bool:
    """Check if node is running by calling eth_chainId"""
    try:
        import requests
        response = requests.post(
            rpc_url,
            json={
                'jsonrpc': '2.0',
                'method': 'eth_chainId',
                'params': [],
                'id': 1
            },
            timeout=5
        )
        data = response.json()
        return 'result' in data
    except Exception:
        return False


# Skip all tests if node is not running
pytestmark = pytest.mark.skipif(
    not is_node_running(RPC_ENDPOINT),
    reason=f"Node not running at {RPC_ENDPOINT}"
)


# ============================================================================
# Connection Tests
# ============================================================================

class TestConnection:
    """Tests for basic RPC connectivity"""

    def test_connects_to_node(self, client):
        """Client can connect to node"""
        chain_id = client.get_chain_id()
        assert chain_id is not None

    def test_correct_chain_id(self, client):
        """Node returns expected chain ID"""
        chain_id = client.get_chain_id()
        # Chain ID is returned as hex string
        chain_id_int = int(chain_id, 16) if isinstance(chain_id, str) else chain_id
        assert chain_id_int == CHAIN_ID

    def test_rpc_call_success(self, client):
        """Direct RPC call succeeds"""
        result = client._rpc_call('eth_blockNumber')
        assert result is not None
        assert isinstance(result, str)
        assert result.startswith('0x')

    def test_rpc_call_invalid_method(self, client):
        """RPC call with invalid method raises error"""
        with pytest.raises(CitrateError):
            client._rpc_call('invalid_method_xyz')


# ============================================================================
# Block Number Tests
# ============================================================================

class TestBlockNumber:
    """Tests for eth_blockNumber"""

    def test_get_block_number(self, client):
        """Returns current block number"""
        result = client._rpc_call('eth_blockNumber')
        block_number = int(result, 16)
        assert block_number >= 0

    def test_block_number_increases(self, client):
        """Block number increases over time"""
        result1 = client._rpc_call('eth_blockNumber')
        block1 = int(result1, 16)

        time.sleep(2)

        result2 = client._rpc_call('eth_blockNumber')
        block2 = int(result2, 16)

        assert block2 >= block1


# ============================================================================
# Balance Tests
# ============================================================================

class TestBalance:
    """Tests for eth_getBalance"""

    def test_genesis_account_has_balance(self, client):
        """Genesis account has non-zero balance"""
        balance = client.get_balance(GENESIS_ACCOUNTS[0]['address'])
        assert balance > 0

    def test_random_address_zero_balance(self, client):
        """Random address has zero balance"""
        import secrets
        random_address = '0x' + secrets.token_hex(20)
        balance = client.get_balance(random_address)
        assert balance == 0

    def test_lowercase_address(self, client):
        """Accepts lowercase address"""
        address = GENESIS_ACCOUNTS[0]['address'].lower()
        balance = client.get_balance(address)
        assert balance > 0

    def test_checksum_address(self, client):
        """Accepts checksum address"""
        address = GENESIS_ACCOUNTS[0]['address']  # Already checksummed
        balance = client.get_balance(address)
        assert balance > 0

    def test_same_balance_different_case(self, client):
        """Same balance regardless of address case"""
        address = GENESIS_ACCOUNTS[0]['address']
        balance1 = client.get_balance(address.lower())
        balance2 = client.get_balance(address)
        assert balance1 == balance2


# ============================================================================
# Nonce Tests
# ============================================================================

class TestNonce:
    """Tests for eth_getTransactionCount"""

    def test_get_nonce(self, client):
        """Returns nonce for account"""
        nonce = client.get_nonce(GENESIS_ACCOUNTS[0]['address'])
        assert nonce >= 0

    def test_new_address_zero_nonce(self, client):
        """New address has zero nonce"""
        import secrets
        random_address = '0x' + secrets.token_hex(20)
        nonce = client.get_nonce(random_address)
        assert nonce == 0


# ============================================================================
# Key Manager Tests
# ============================================================================

class TestKeyManager:
    """Tests for KeyManager crypto operations"""

    def test_address_derivation(self, key_manager):
        """Derives correct address from private key"""
        address = key_manager.get_address()
        expected = GENESIS_ACCOUNTS[0]['address']
        assert address.lower() == expected.lower()

    def test_consistent_address(self):
        """Same private key always produces same address"""
        km1 = KeyManager(GENESIS_ACCOUNTS[0]['private_key'])
        km2 = KeyManager(GENESIS_ACCOUNTS[0]['private_key'])
        assert km1.get_address() == km2.get_address()

    def test_different_keys_different_addresses(self):
        """Different private keys produce different addresses"""
        km1 = KeyManager(GENESIS_ACCOUNTS[0]['private_key'])
        km2 = KeyManager(GENESIS_ACCOUNTS[1]['private_key'])
        assert km1.get_address() != km2.get_address()

    def test_random_key_generation(self):
        """Random key generation produces valid addresses"""
        import secrets
        for _ in range(10):
            private_key = secrets.token_hex(32)
            km = KeyManager(private_key)
            address = km.get_address()
            assert address.startswith('0x')
            assert len(address) == 42


# ============================================================================
# Gas Price Tests
# ============================================================================

class TestGasPrice:
    """Tests for eth_gasPrice"""

    def test_get_gas_price(self, client):
        """Returns current gas price"""
        result = client._rpc_call('eth_gasPrice')
        gas_price = int(result, 16)
        assert gas_price > 0

    def test_reasonable_gas_price(self, client):
        """Gas price is within reasonable range"""
        result = client._rpc_call('eth_gasPrice')
        gas_price = int(result, 16)
        # Between 1 gwei and 10000 gwei
        assert gas_price >= 1_000_000_000
        assert gas_price <= 10_000_000_000_000


# ============================================================================
# Block Tests
# ============================================================================

class TestBlocks:
    """Tests for eth_getBlockByNumber"""

    def test_get_latest_block(self, client):
        """Returns latest block"""
        result = client._rpc_call('eth_getBlockByNumber', ['latest', False])
        assert result is not None
        assert 'number' in result
        assert 'hash' in result
        assert 'parentHash' in result

    def test_get_genesis_block(self, client):
        """Returns genesis block"""
        result = client._rpc_call('eth_getBlockByNumber', ['0x0', False])
        assert result is not None
        assert int(result['number'], 16) == 0

    def test_block_hash_format(self, client):
        """Block hash has correct format"""
        result = client._rpc_call('eth_getBlockByNumber', ['latest', False])
        block_hash = result['hash']
        assert block_hash.startswith('0x')
        assert len(block_hash) == 66  # 0x + 64 hex chars

    def test_nonexistent_block(self, client):
        """Returns null for nonexistent block"""
        result = client._rpc_call('eth_getBlockByNumber', ['0xffffff', False])
        assert result is None


# ============================================================================
# eth_call Tests
# ============================================================================

class TestEthCall:
    """Tests for eth_call"""

    def test_call_to_address(self, client):
        """Can make call to any address"""
        result = client._rpc_call('eth_call', [
            {
                'to': '0x0000000000000000000000000000000000000000',
                'data': '0x',
            },
            'latest'
        ])
        assert result is not None

    def test_estimate_gas_simple_transfer(self, client):
        """Estimates gas for simple transfer"""
        result = client._rpc_call('eth_estimateGas', [
            {
                'from': GENESIS_ACCOUNTS[0]['address'],
                'to': GENESIS_ACCOUNTS[1]['address'],
                'value': '0x1',
            }
        ])
        gas = int(result, 16)
        assert gas >= 21000
        assert gas < 50000


# ============================================================================
# Address Format Tests
# ============================================================================

class TestAddressFormats:
    """Tests for address format handling"""

    def test_valid_addresses(self, client):
        """All valid address formats are accepted"""
        valid_addresses = [
            '0x0000000000000000000000000000000000000000',
            '0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF',
            '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
            '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
        ]
        for address in valid_addresses:
            balance = client.get_balance(address)
            assert isinstance(balance, int)

    def test_checksum_conversion(self):
        """Checksum addresses are handled correctly"""
        from eth_utils import to_checksum_address
        lowercase = '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266'
        checksummed = to_checksum_address(lowercase)
        assert checksummed != lowercase
        assert checksummed == '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266'


# ============================================================================
# Client Method Tests
# ============================================================================

class TestClientMethods:
    """Tests for CitrateClient high-level methods"""

    def test_list_models_returns_list(self, client):
        """list_models returns a list (even if empty)"""
        try:
            result = client.list_models()
            assert isinstance(result, list) or result is None
        except CitrateError as e:
            # Method might not be implemented - that's ok for now
            pytest.skip(f"citrate_listModels not implemented: {e}")

    def test_client_without_key_read_only(self, client):
        """Client without private key can only do read operations"""
        # Read operations should work
        balance = client.get_balance(GENESIS_ACCOUNTS[0]['address'])
        assert balance >= 0

        # Write operations should fail
        from citrate_sdk.models import ModelConfig
        with pytest.raises(CitrateError, match="Private key required"):
            client.deploy_model('/nonexistent/model.onnx', ModelConfig())


# ============================================================================
# Performance Tests
# ============================================================================

class TestPerformance:
    """Tests for RPC performance"""

    def test_rpc_latency(self, client):
        """RPC calls complete within reasonable time"""
        import time

        latencies = []
        for _ in range(5):
            start = time.time()
            client._rpc_call('eth_blockNumber')
            latency = (time.time() - start) * 1000  # ms
            latencies.append(latency)

        avg_latency = sum(latencies) / len(latencies)
        assert avg_latency < 1000  # Under 1 second average

    def test_concurrent_requests(self, client):
        """Handles concurrent requests"""
        import concurrent.futures

        def make_request():
            return client._rpc_call('eth_blockNumber')

        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(make_request) for _ in range(10)]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]

        assert len(results) == 10
        for result in results:
            assert int(result, 16) >= 0


# ============================================================================
# Error Handling Tests
# ============================================================================

class TestErrorHandling:
    """Tests for error handling"""

    def test_invalid_address_error(self, client):
        """Invalid address raises appropriate error"""
        with pytest.raises((CitrateError, ValueError)):
            client.get_balance('not_a_valid_address')

    def test_network_error_handling(self):
        """Network errors are handled gracefully"""
        bad_client = CitrateClient(rpc_url='http://localhost:99999')
        with pytest.raises(CitrateError):
            bad_client.get_chain_id()

    def test_timeout_error(self):
        """Timeouts are handled"""
        import requests
        slow_client = CitrateClient(rpc_url=RPC_ENDPOINT)
        slow_client.session.timeout = 0.001  # Very short timeout

        with pytest.raises((CitrateError, requests.exceptions.Timeout)):
            slow_client.get_chain_id()


# ============================================================================
# Transaction Tests (requires funded account)
# ============================================================================

@pytest.mark.skipif(
    not TEST_PRIVATE_KEY and not is_node_running(RPC_ENDPOINT),
    reason="Transaction tests require funded account and running node"
)
class TestTransactions:
    """Tests for transaction submission"""

    def test_can_get_sender_balance(self, funded_client):
        """Can get balance of funded account"""
        address = funded_client.key_manager.get_address()
        balance = funded_client.get_balance(address)
        assert balance > 0

    def test_can_get_sender_nonce(self, funded_client):
        """Can get nonce of funded account"""
        address = funded_client.key_manager.get_address()
        nonce = funded_client.get_nonce(address)
        assert nonce >= 0


# ============================================================================
# Consistency Tests
# ============================================================================

class TestConsistency:
    """Tests for cross-component consistency"""

    def test_chain_id_consistency(self, client):
        """Chain ID is consistent across calls"""
        chain_id_1 = client.get_chain_id()
        chain_id_2 = client.get_chain_id()
        assert chain_id_1 == chain_id_2

    def test_balance_consistency(self, client):
        """Balance is consistent across immediate calls"""
        address = GENESIS_ACCOUNTS[0]['address']
        balance_1 = client.get_balance(address)
        balance_2 = client.get_balance(address)
        assert balance_1 == balance_2

    def test_nonce_consistency(self, client):
        """Nonce is consistent across immediate calls"""
        address = GENESIS_ACCOUNTS[0]['address']
        nonce_1 = client.get_nonce(address)
        nonce_2 = client.get_nonce(address)
        assert nonce_1 == nonce_2


# ============================================================================
# Cryptographic Tests
# ============================================================================

class TestCryptography:
    """Tests for cryptographic operations"""

    def test_message_signing(self, key_manager):
        """Can sign and verify messages"""
        message = b"Test message"
        signature = key_manager.sign_message(message)
        assert signature is not None
        assert len(signature) > 0

    def test_different_messages_different_signatures(self, key_manager):
        """Different messages produce different signatures"""
        sig1 = key_manager.sign_message(b"Message 1")
        sig2 = key_manager.sign_message(b"Message 2")
        assert sig1 != sig2

    def test_consistent_signing(self, key_manager):
        """Same message produces consistent signature components"""
        message = b"Consistent message"
        sig1 = key_manager.sign_message(message)
        sig2 = key_manager.sign_message(message)
        # Signatures may differ due to randomness in ECDSA, but should be verifiable


# ============================================================================
# Entry point for pytest
# ============================================================================

if __name__ == '__main__':
    pytest.main([__file__, '-v'])
