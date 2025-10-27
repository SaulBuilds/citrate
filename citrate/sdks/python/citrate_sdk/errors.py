"""
Exception classes for Citrate SDK
"""


class CitrateError(Exception):
    """Base exception for Citrate SDK errors"""

    def __init__(self, message: str, error_code: str = None, details: dict = None):
        super().__init__(message)
        self.error_code = error_code
        self.details = details or {}


class NetworkError(CitrateError):
    """Network communication errors"""
    pass


class AuthenticationError(CitrateError):
    """Authentication and authorization errors"""
    pass


class ModelNotFoundError(CitrateError):
    """Model not found or doesn't exist"""
    pass


class InsufficientFundsError(CitrateError):
    """Insufficient funds for operation"""
    pass


class ModelDeploymentError(CitrateError):
    """Model deployment failed"""
    pass


class InferenceError(CitrateError):
    """Inference execution failed"""
    pass


class EncryptionError(CitrateError):
    """Encryption/decryption failed"""
    pass


class ValidationError(CitrateError):
    """Input validation failed"""
    pass


class TimeoutError(CitrateError):
    """Operation timed out"""
    pass


class ConfigurationError(CitrateError):
    """SDK configuration error"""
    pass


class IPFSError(CitrateError):
    """IPFS storage error"""
    pass


class ContractError(CitrateError):
    """Smart contract execution error"""
    pass


class IPFSError(CitrateError):
    """IPFS storage error"""
    pass