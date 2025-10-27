/**
 * Constants for Citrate JavaScript SDK
 */

// Network constants
export const CHAIN_IDS = {
  MAINNET: 1,
  TESTNET: 1337,
  LOCAL: 31337
} as const;

export const DEFAULT_RPC_URLS = {
  [CHAIN_IDS.MAINNET]: 'https://mainnet.citrate.ai',
  [CHAIN_IDS.TESTNET]: 'https://testnet.citrate.ai',
  [CHAIN_IDS.LOCAL]: 'http://localhost:8545'
} as const;

export const DEFAULT_WS_URLS = {
  [CHAIN_IDS.MAINNET]: 'wss://mainnet.citrate.ai/ws',
  [CHAIN_IDS.TESTNET]: 'wss://testnet.citrate.ai/ws',
  [CHAIN_IDS.LOCAL]: 'ws://localhost:8546'
} as const;

// Precompile addresses
export const PRECOMPILE_ADDRESSES = {
  MODEL_DEPLOY: '0x0100000000000000000000000000000000000100',
  MODEL_INFERENCE: '0x0100000000000000000000000000000000000101',
  MODEL_REGISTRY: '0x0100000000000000000000000000000000000102',
  MODEL_METADATA: '0x0100000000000000000000000000000000000103',
  ACCESS_CONTROL: '0x0100000000000000000000000000000000000104',
  INFERENCE_CACHE: '0x0100000000000000000000000000000000000105',
  MODEL_ENCRYPTION: '0x0100000000000000000000000000000000000106'
} as const;

// Gas limits
export const GAS_LIMITS = {
  MODEL_DEPLOY: 500000n,
  INFERENCE: 1000000n,
  ACCESS_PURCHASE: 200000n,
  METADATA_UPDATE: 100000n,
  REGISTRY_UPDATE: 150000n
} as const;

// Timeout values (milliseconds)
export const TIMEOUTS = {
  DEFAULT_REQUEST: 30000,
  MODEL_DEPLOYMENT: 300000, // 5 minutes
  INFERENCE_EXECUTION: 120000, // 2 minutes
  WEBSOCKET_CONNECTION: 10000,
  IPFS_UPLOAD: 60000
} as const;

// Model constraints
export const MODEL_LIMITS = {
  MAX_MODEL_SIZE: 100 * 1024 * 1024, // 100MB
  MAX_BATCH_SIZE: 100,
  MAX_INPUT_SIZE: 10 * 1024 * 1024, // 10MB
  MAX_OUTPUT_SIZE: 10 * 1024 * 1024, // 10MB
  MAX_METADATA_SIZE: 64 * 1024, // 64KB
  MAX_DESCRIPTION_LENGTH: 1000,
  MAX_NAME_LENGTH: 100,
  MAX_TAGS: 10,
  MAX_TAG_LENGTH: 20
} as const;

// Encryption settings
export const ENCRYPTION = {
  ALGORITHM: 'AES-256-GCM',
  KEY_DERIVATION: 'HKDF-SHA256',
  NONCE_SIZE: 12,
  AUTH_TAG_SIZE: 16,
  KEY_SIZE: 32,
  SALT_SIZE: 16,
  PBKDF2_ITERATIONS: 10000
} as const;

// Event names
export const EVENTS = {
  // Connection events
  CONNECTED: 'connected',
  DISCONNECTED: 'disconnected',
  ERROR: 'error',

  // Model events
  MODEL_DEPLOYED: 'modelDeployed',
  MODEL_UPDATED: 'modelUpdated',
  MODEL_DELETED: 'modelDeleted',

  // Inference events
  INFERENCE_STARTED: 'inferenceStarted',
  INFERENCE_PARTIAL: 'inferencePartial',
  INFERENCE_COMPLETED: 'inferenceCompleted',
  INFERENCE_FAILED: 'inferenceFailed',

  // Marketplace events
  MARKETPLACE_SALE: 'marketplaceSale',
  MARKETPLACE_LISTING: 'marketplaceListing',
  ACCESS_GRANTED: 'accessGranted',

  // Payment events
  PAYMENT_RECEIVED: 'paymentReceived',
  REVENUE_DISTRIBUTED: 'revenueDistributed'
} as const;

// Model types mapping
export const MODEL_TYPE_EXTENSIONS = {
  'coreml': ['.mlpackage', '.mlmodel'],
  'onnx': ['.onnx'],
  'tensorflow': ['.pb', '.savedmodel'],
  'pytorch': ['.pt', '.pth', '.pkl'],
  'custom': ['.json', '.bin', '.dat']
} as const;

// HTTP status codes
export const HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  CONFLICT: 409,
  INTERNAL_SERVER_ERROR: 500,
  BAD_GATEWAY: 502,
  SERVICE_UNAVAILABLE: 503
} as const;

// WebSocket close codes
export const WS_CLOSE_CODES = {
  NORMAL_CLOSURE: 1000,
  GOING_AWAY: 1001,
  PROTOCOL_ERROR: 1002,
  UNSUPPORTED_DATA: 1003,
  INVALID_FRAME_PAYLOAD_DATA: 1007,
  POLICY_VIOLATION: 1008,
  MESSAGE_TOO_BIG: 1009,
  INTERNAL_ERROR: 1011
} as const;

// API endpoints
export const API_ENDPOINTS = {
  MODELS: '/v1/models',
  INFERENCE: '/v1/inference',
  MARKETPLACE: '/v1/marketplace',
  CHAT_COMPLETIONS: '/v1/chat/completions',
  EMBEDDINGS: '/v1/embeddings',
  JOBS: '/v1/jobs',
  MESSAGES: '/v1/messages'
} as const;