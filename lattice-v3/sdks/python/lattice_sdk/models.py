"""
Data models for Lattice SDK
"""

from dataclasses import dataclass, field
from typing import Dict, Any, List, Optional
from enum import Enum


class ModelType(Enum):
    """Supported model types"""
    COREML = "coreml"
    ONNX = "onnx"
    TENSORFLOW = "tensorflow"
    PYTORCH = "pytorch"
    CUSTOM = "custom"


class AccessType(Enum):
    """Model access types"""
    PUBLIC = "public"
    PRIVATE = "private"
    PAID = "paid"
    WHITELIST = "whitelist"


@dataclass
class EncryptionConfig:
    """Configuration for model encryption"""
    enabled: bool = True
    algorithm: str = "AES-256-GCM"
    key_derivation: str = "HKDF-SHA256"
    access_control: bool = True
    threshold_shares: int = 3
    total_shares: int = 5


@dataclass
class ModelConfig:
    """Configuration for model deployment"""
    # Basic settings
    name: str = ""
    description: str = ""
    model_type: ModelType = ModelType.COREML
    version: str = "1.0.0"

    # Access control
    access_type: AccessType = AccessType.PUBLIC
    access_price: int = 0  # Price in wei per inference
    access_list: Optional[List[str]] = None  # Whitelist addresses

    # Encryption
    encrypted: bool = False
    encryption_config: Optional[EncryptionConfig] = None

    # Metadata
    metadata: Optional[Dict[str, Any]] = None
    tags: List[str] = field(default_factory=list)

    # Performance
    max_batch_size: int = 1
    timeout_seconds: int = 30
    memory_limit_mb: int = 1024

    # Revenue sharing
    revenue_shares: Optional[Dict[str, float]] = None  # address -> percentage


@dataclass
class ModelDeployment:
    """Result of model deployment"""
    model_id: str
    tx_hash: str
    ipfs_hash: str
    encrypted: bool
    access_price: int
    deployment_time: int
    gas_used: Optional[int] = None
    deployment_cost: Optional[int] = None


@dataclass
class InferenceRequest:
    """Request for model inference"""
    model_id: str
    input_data: Dict[str, Any]
    encrypted: bool = False
    batch_size: int = 1
    timeout: int = 30
    timestamp: Optional[int] = None


@dataclass
class InferenceResult:
    """Result of model inference"""
    model_id: str
    output_data: Dict[str, Any]
    gas_used: int
    execution_time: float  # milliseconds
    tx_hash: str
    confidence: Optional[float] = None
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class ModelInfo:
    """Detailed model information"""
    model_id: str
    name: str
    description: str
    owner: str
    model_type: ModelType
    access_type: AccessType
    access_price: int
    encrypted: bool
    ipfs_hash: str
    deployment_time: int
    total_inferences: int
    total_revenue: int
    metadata: Dict[str, Any]
    tags: List[str]


@dataclass
class ModelStats:
    """Model usage statistics"""
    model_id: str
    total_inferences: int
    total_revenue: int
    average_execution_time: float
    average_gas_cost: int
    unique_users: int
    last_inference_time: int


@dataclass
class PaymentInfo:
    """Payment information for model access"""
    model_id: str
    price_per_inference: int
    payment_token: str = "ETH"
    payment_address: str = ""
    revenue_sharing: Optional[Dict[str, float]] = None


@dataclass
class AccessControlEntry:
    """Access control entry for a model"""
    address: str
    access_level: str  # "read", "write", "admin"
    granted_by: str
    granted_at: int
    expires_at: Optional[int] = None


@dataclass
class ModelVersion:
    """Model version information"""
    model_id: str
    version: str
    ipfs_hash: str
    deployment_time: int
    changes: str
    deprecated: bool = False