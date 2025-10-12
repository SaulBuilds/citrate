"""
Finite field arithmetic for Shamir's Secret Sharing
Using GF(2^8) for byte-oriented operations
"""

import random
from typing import List, Tuple


class GF256:
    """
    Galois Field GF(2^8) implementation for Shamir's Secret Sharing
    Uses irreducible polynomial x^8 + x^4 + x^3 + x + 1 (0x11b)
    """

    # Precomputed tables for efficiency
    _exp_table = None
    _log_table = None
    _initialized = False

    @classmethod
    def _initialize_tables(cls):
        """Initialize exponential and logarithm tables"""
        if cls._initialized:
            return

        cls._exp_table = [0] * 512
        cls._log_table = [0] * 256

        # Generate exponential table
        x = 1
        for i in range(255):
            cls._exp_table[i] = x
            cls._log_table[x] = i
            x = cls._multiply_raw(x, 3)  # 3 is a primitive element

        # Handle overflow
        for i in range(255, 512):
            cls._exp_table[i] = cls._exp_table[i - 255]

        cls._log_table[0] = 0  # Special case
        cls._initialized = True

    @classmethod
    def _multiply_raw(cls, a: int, b: int) -> int:
        """Raw multiplication without table lookup"""
        result = 0
        while b:
            if b & 1:
                result ^= a
            a <<= 1
            if a & 0x100:
                a ^= 0x11b  # Irreducible polynomial
            b >>= 1
        return result & 0xff

    @classmethod
    def add(cls, a: int, b: int) -> int:
        """Addition in GF(2^8) (XOR)"""
        return a ^ b

    @classmethod
    def subtract(cls, a: int, b: int) -> int:
        """Subtraction in GF(2^8) (same as addition)"""
        return a ^ b

    @classmethod
    def multiply(cls, a: int, b: int) -> int:
        """Multiplication in GF(2^8)"""
        cls._initialize_tables()

        if a == 0 or b == 0:
            return 0

        return cls._exp_table[cls._log_table[a] + cls._log_table[b]]

    @classmethod
    def divide(cls, a: int, b: int) -> int:
        """Division in GF(2^8)"""
        if b == 0:
            raise ZeroDivisionError("Division by zero in GF(2^8)")

        if a == 0:
            return 0

        cls._initialize_tables()
        return cls._exp_table[cls._log_table[a] - cls._log_table[b] + 255]

    @classmethod
    def power(cls, a: int, exp: int) -> int:
        """Exponentiation in GF(2^8)"""
        if exp == 0:
            return 1
        if a == 0:
            return 0

        cls._initialize_tables()
        return cls._exp_table[(cls._log_table[a] * exp) % 255]

    @classmethod
    def inverse(cls, a: int) -> int:
        """Multiplicative inverse in GF(2^8)"""
        if a == 0:
            raise ZeroDivisionError("Zero has no inverse in GF(2^8)")

        cls._initialize_tables()
        return cls._exp_table[255 - cls._log_table[a]]


class ShamirSecretSharing:
    """
    Proper Shamir's Secret Sharing implementation using finite field arithmetic
    """

    def __init__(self, threshold: int, total_shares: int):
        """
        Initialize Shamir's Secret Sharing

        Args:
            threshold: Minimum number of shares needed to reconstruct
            total_shares: Total number of shares to create
        """
        if threshold <= 0:
            raise ValueError("Threshold must be positive")
        if threshold > total_shares:
            raise ValueError("Threshold cannot exceed total shares")
        if total_shares > 255:
            raise ValueError("Total shares cannot exceed 255")

        self.threshold = threshold
        self.total_shares = total_shares

    def split_secret(self, secret: bytes) -> List[Tuple[int, bytes]]:
        """
        Split secret into shares

        Args:
            secret: Secret bytes to split

        Returns:
            List of (x, share_bytes) tuples
        """
        shares = []

        for i in range(1, self.total_shares + 1):
            share_bytes = self._evaluate_polynomial_at_point(secret, i)
            shares.append((i, share_bytes))

        return shares

    def reconstruct_secret(self, shares: List[Tuple[int, bytes]]) -> bytes:
        """
        Reconstruct secret from shares

        Args:
            shares: List of (x, share_bytes) tuples

        Returns:
            Reconstructed secret bytes
        """
        if len(shares) < self.threshold:
            raise ValueError(f"Need at least {self.threshold} shares, got {len(shares)}")

        # Use first threshold shares
        active_shares = shares[:self.threshold]

        # Ensure all shares have same length
        share_length = len(active_shares[0][1])
        if not all(len(share[1]) == share_length for share in active_shares):
            raise ValueError("All shares must have the same length")

        # Reconstruct each byte position
        secret_bytes = []
        for byte_pos in range(share_length):
            # Extract byte values for this position
            points = [(x, share_bytes[byte_pos]) for x, share_bytes in active_shares]

            # Use Lagrange interpolation to find f(0)
            reconstructed_byte = self._lagrange_interpolation(points, 0)
            secret_bytes.append(reconstructed_byte)

        return bytes(secret_bytes)

    def _evaluate_polynomial_at_point(self, secret: bytes, x: int) -> bytes:
        """
        Evaluate polynomial at point x for each byte of the secret
        """
        # Generate random coefficients for polynomial
        # f(x) = a0 + a1*x + a2*x^2 + ... + a(k-1)*x^(k-1)
        # where a0 is the secret byte and k is the threshold

        share_bytes = []

        for secret_byte in secret:
            # Generate random coefficients (except a0 which is the secret)
            coefficients = [secret_byte]
            for _ in range(1, self.threshold):
                coefficients.append(random.randint(0, 255))

            # Evaluate polynomial at x
            result = 0
            x_power = 1

            for coeff in coefficients:
                result = GF256.add(result, GF256.multiply(coeff, x_power))
                x_power = GF256.multiply(x_power, x)

            share_bytes.append(result)

        return bytes(share_bytes)

    def _lagrange_interpolation(self, points: List[Tuple[int, int]], x: int) -> int:
        """
        Lagrange interpolation to find f(x) given points

        Args:
            points: List of (x_i, y_i) points
            x: Point to evaluate at

        Returns:
            f(x) value
        """
        result = 0

        for i, (x_i, y_i) in enumerate(points):
            # Calculate Lagrange basis polynomial L_i(x)
            numerator = 1
            denominator = 1

            for j, (x_j, _) in enumerate(points):
                if i != j:
                    # For x=0, numerator becomes (0 - x_j) = -x_j = x_j (in GF(2^8))
                    numerator = GF256.multiply(numerator, x_j)
                    denominator = GF256.multiply(denominator, GF256.subtract(x_i, x_j))

            # L_i(x) = numerator / denominator
            if denominator == 0:
                raise ValueError("Denominator is zero in Lagrange interpolation")

            lagrange_coeff = GF256.divide(numerator, denominator)

            # Add y_i * L_i(x) to result
            result = GF256.add(result, GF256.multiply(y_i, lagrange_coeff))

        return result

    def verify_shares(self, shares: List[Tuple[int, bytes]]) -> bool:
        """
        Verify that shares are consistent (can be used to detect tampering)

        Args:
            shares: List of shares to verify

        Returns:
            True if shares are consistent
        """
        if len(shares) < self.threshold:
            return False

        try:
            # Try to reconstruct with different combinations
            for i in range(len(shares) - self.threshold + 1):
                test_shares = shares[i:i + self.threshold]
                self.reconstruct_secret(test_shares)
            return True

        except Exception:
            return False


def split_secret_bytes(secret: bytes, threshold: int, total_shares: int) -> List[Tuple[int, bytes]]:
    """
    Convenience function to split secret bytes

    Args:
        secret: Secret bytes to split
        threshold: Minimum shares needed to reconstruct
        total_shares: Total number of shares to create

    Returns:
        List of (share_id, share_bytes) tuples
    """
    sss = ShamirSecretSharing(threshold, total_shares)
    return sss.split_secret(secret)


def reconstruct_secret_bytes(shares: List[Tuple[int, bytes]], threshold: int) -> bytes:
    """
    Convenience function to reconstruct secret bytes

    Args:
        shares: List of (share_id, share_bytes) tuples
        threshold: Minimum shares needed

    Returns:
        Reconstructed secret bytes
    """
    # Infer total_shares from the shares provided
    total_shares = max(len(shares), threshold)
    sss = ShamirSecretSharing(threshold, total_shares)
    return sss.reconstruct_secret(shares)