"""
Citrate Python SDK

A comprehensive Python SDK for interacting with the Citrate AI blockchain platform.
Provides easy-to-use interfaces for model deployment, inference execution,
encryption, access control, and payment systems.
"""

from .client import CitrateClient
from .models import ModelConfig, ModelDeployment, InferenceRequest, InferenceResult, ModelType, AccessType
from .crypto import EncryptionConfig, KeyManager
from .errors import CitrateError, ModelNotFoundError, InsufficientFundsError

__version__ = "0.1.0"
__author__ = "Citrate Team"

__all__ = [
    "CitrateClient",
    "ModelConfig",
    "ModelDeployment",
    "InferenceRequest",
    "InferenceResult",
    "ModelType",
    "AccessType",
    "EncryptionConfig",
    "KeyManager",
    "CitrateError",
    "ModelNotFoundError",
    "InsufficientFundsError"
]