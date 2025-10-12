"""
Exception classes for Lattice SDK
"""


class LatticeError(Exception):
    """Base exception for Lattice SDK errors"""

    def __init__(self, message: str, error_code: str = None, details: dict = None):
        super().__init__(message)
        self.error_code = error_code
        self.details = details or {}


class NetworkError(LatticeError):
    """Network communication errors"""
    pass


class AuthenticationError(LatticeError):
    """Authentication and authorization errors"""
    pass


class ModelNotFoundError(LatticeError):
    """Model not found or doesn't exist"""
    pass


class InsufficientFundsError(LatticeError):
    """Insufficient funds for operation"""
    pass


class ModelDeploymentError(LatticeError):
    """Model deployment failed"""
    pass


class InferenceError(LatticeError):
    """Inference execution failed"""
    pass


class EncryptionError(LatticeError):
    """Encryption/decryption failed"""
    pass


class ValidationError(LatticeError):
    """Input validation failed"""
    pass


class TimeoutError(LatticeError):
    """Operation timed out"""
    pass


class ConfigurationError(LatticeError):
    """SDK configuration error"""
    pass


class IPFSError(LatticeError):
    """IPFS storage error"""
    pass


class ContractError(LatticeError):
    """Smart contract execution error"""
    pass


class IPFSError(LatticeError):
    """IPFS storage error"""
    pass