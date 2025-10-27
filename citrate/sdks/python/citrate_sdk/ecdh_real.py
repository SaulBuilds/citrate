"""
Real ECDH implementation using proper elliptic curve cryptography
"""

from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.backends import default_backend
from .errors import CitrateError


class ECDHManager:
    """
    Real ECDH key exchange using secp256k1 elliptic curve
    """

    def __init__(self, private_key_bytes: bytes = None):
        """
        Initialize ECDH manager

        Args:
            private_key_bytes: Optional 32-byte private key, generates new if None
        """
        if private_key_bytes:
            # Load existing private key
            if len(private_key_bytes) != 32:
                raise CitrateError("Private key must be 32 bytes")

            # Convert bytes to EC private key
            private_value = int.from_bytes(private_key_bytes, 'big')
            self.private_key = ec.derive_private_key(private_value, ec.SECP256K1(), default_backend())
        else:
            # Generate new private key
            self.private_key = ec.generate_private_key(ec.SECP256K1(), default_backend())

        self.public_key = self.private_key.public_key()

    def get_private_key_bytes(self) -> bytes:
        """
        Get private key as 32 bytes

        Returns:
            Private key bytes
        """
        private_value = self.private_key.private_numbers().private_value
        return private_value.to_bytes(32, 'big')

    def get_public_key_compressed(self) -> bytes:
        """
        Get compressed public key (33 bytes)

        Returns:
            Compressed public key bytes
        """
        return self.public_key.public_numbers().x.to_bytes(32, 'big')

    def get_public_key_uncompressed(self) -> bytes:
        """
        Get uncompressed public key (65 bytes)

        Returns:
            Uncompressed public key bytes
        """
        public_numbers = self.public_key.public_numbers()
        x_bytes = public_numbers.x.to_bytes(32, 'big')
        y_bytes = public_numbers.y.to_bytes(32, 'big')
        return b'\x04' + x_bytes + y_bytes

    def perform_ecdh(self, peer_public_key_bytes: bytes) -> bytes:
        """
        Perform ECDH key exchange with peer's public key

        Args:
            peer_public_key_bytes: Peer's public key (32 or 33 or 65 bytes)

        Returns:
            32-byte shared secret

        Raises:
            CitrateError: If ECDH fails
        """
        try:
            # Parse peer public key based on length
            if len(peer_public_key_bytes) == 32:
                # Compressed x-coordinate only
                x = int.from_bytes(peer_public_key_bytes, 'big')
                y = self._recover_y_coordinate(x)
                peer_public_numbers = ec.EllipticCurvePublicNumbers(x, y, ec.SECP256K1())

            elif len(peer_public_key_bytes) == 33:
                # Compressed format with prefix
                if peer_public_key_bytes[0] not in [0x02, 0x03]:
                    raise CitrateError("Invalid compressed public key prefix")

                x = int.from_bytes(peer_public_key_bytes[1:], 'big')
                y = self._recover_y_coordinate(x, peer_public_key_bytes[0] == 0x03)
                peer_public_numbers = ec.EllipticCurvePublicNumbers(x, y, ec.SECP256K1())

            elif len(peer_public_key_bytes) == 65:
                # Uncompressed format
                if peer_public_key_bytes[0] != 0x04:
                    raise CitrateError("Invalid uncompressed public key prefix")

                x = int.from_bytes(peer_public_key_bytes[1:33], 'big')
                y = int.from_bytes(peer_public_key_bytes[33:65], 'big')
                peer_public_numbers = ec.EllipticCurvePublicNumbers(x, y, ec.SECP256K1())

            else:
                raise CitrateError(f"Invalid public key length: {len(peer_public_key_bytes)}")

            # Create public key object
            peer_public_key = peer_public_numbers.public_key(default_backend())

            # Perform ECDH
            shared_key = self.private_key.exchange(ec.ECDH(), peer_public_key)

            return shared_key

        except Exception as e:
            raise CitrateError(f"ECDH key exchange failed: {str(e)}")

    def derive_shared_secret(self, peer_public_key_bytes: bytes,
                           salt: bytes = b"citrate-ecdh",
                           info: bytes = b"shared-secret") -> bytes:
        """
        Perform ECDH and derive a shared secret using HKDF

        Args:
            peer_public_key_bytes: Peer's public key bytes
            salt: Salt for HKDF
            info: Info parameter for HKDF

        Returns:
            32-byte derived shared secret
        """
        # Perform ECDH
        shared_key = self.perform_ecdh(peer_public_key_bytes)

        # Derive final secret using HKDF
        hkdf = HKDF(
            algorithm=hashes.SHA256(),
            length=32,
            salt=salt,
            info=info,
            backend=default_backend()
        )

        return hkdf.derive(shared_key)

    def _recover_y_coordinate(self, x: int, is_odd: bool = False) -> int:
        """
        Recover y coordinate from x coordinate for secp256k1 curve

        Args:
            x: X coordinate
            is_odd: Whether to use odd y coordinate

        Returns:
            Y coordinate
        """
        # secp256k1 parameters
        p = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
        a = 0
        b = 7

        # Calculate y^2 = x^3 + ax + b (mod p)
        y_squared = (pow(x, 3, p) + a * x + b) % p

        # Calculate y = sqrt(y^2) (mod p)
        y = pow(y_squared, (p + 1) // 4, p)

        # Choose correct sign based on is_odd
        if (y % 2) != is_odd:
            y = p - y

        return y

    @staticmethod
    def generate_keypair() -> tuple['ECDHManager', bytes]:
        """
        Generate a new ECDH keypair

        Returns:
            Tuple of (ECDHManager instance, public key bytes)
        """
        ecdh = ECDHManager()
        public_key = ecdh.get_public_key_compressed()
        return ecdh, public_key

    def sign_message(self, message: bytes) -> bytes:
        """
        Sign a message using the private key

        Args:
            message: Message to sign

        Returns:
            DER-encoded signature
        """
        signature = self.private_key.sign(message, ec.ECDSA(hashes.SHA256()))
        return signature

    def verify_signature(self, message: bytes, signature: bytes,
                        public_key_bytes: bytes) -> bool:
        """
        Verify a signature

        Args:
            message: Original message
            signature: DER-encoded signature
            public_key_bytes: Public key of signer

        Returns:
            True if signature is valid
        """
        try:
            # Parse public key
            if len(public_key_bytes) == 33:
                x = int.from_bytes(public_key_bytes[1:], 'big')
                y = self._recover_y_coordinate(x, public_key_bytes[0] == 0x03)
            elif len(public_key_bytes) == 65:
                x = int.from_bytes(public_key_bytes[1:33], 'big')
                y = int.from_bytes(public_key_bytes[33:65], 'big')
            else:
                return False

            public_numbers = ec.EllipticCurvePublicNumbers(x, y, ec.SECP256K1())
            public_key = public_numbers.public_key(default_backend())

            # Verify signature
            public_key.verify(signature, message, ec.ECDSA(hashes.SHA256()))
            return True

        except Exception:
            return False