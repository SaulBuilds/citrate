// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title ModelAccessControl
 * @notice Manages access control and permissions for AI models on Lattice
 * @dev Integrates with precompiles at 0x0100-0x0106 for AI operations
 */
contract ModelAccessControl is Ownable, ReentrancyGuard {
    using ECDSA for bytes32;

    // ============ Constants ============

    // Precompile addresses
    address constant MODEL_DEPLOY = address(0x0100);
    address constant MODEL_INFERENCE = address(0x0101);
    address constant MODEL_ENCRYPTION = address(0x0106);

    // Access levels
    uint8 constant ACCESS_NONE = 0;
    uint8 constant ACCESS_INFERENCE = 1;
    uint8 constant ACCESS_FULL = 2;
    uint8 constant ACCESS_ADMIN = 3;

    // ============ Structs ============

    struct ModelInfo {
        address owner;
        bytes32 modelHash;
        string ipfsCid;
        uint256 createdAt;
        uint256 lastUpdated;
        bool isEncrypted;
        uint256 accessPrice;
        uint256 totalInferences;
    }

    struct AccessGrant {
        uint8 level;
        uint256 expiresAt;
        uint256 usageLimit;
        uint256 usageCount;
        bool revoked;
    }

    struct AccessRequest {
        address requester;
        bytes32 modelId;
        uint8 requestedLevel;
        uint256 payment;
        string reason;
        uint256 timestamp;
        bool approved;
    }

    // ============ State Variables ============

    // Model registry
    mapping(bytes32 => ModelInfo) public models;

    // Access control: modelId => user => AccessGrant
    mapping(bytes32 => mapping(address => AccessGrant)) public accessGrants;

    // Access requests
    mapping(uint256 => AccessRequest) public accessRequests;
    uint256 public nextRequestId;

    // Revenue sharing
    mapping(bytes32 => uint256) public modelRevenue;
    mapping(address => uint256) public pendingWithdrawals;

    // Staking requirements
    mapping(bytes32 => uint256) public stakingRequirements;
    mapping(address => mapping(bytes32 => uint256)) public userStakes;

    // Model categories and metadata
    mapping(bytes32 => string) public modelCategories;
    mapping(bytes32 => string) public modelDescriptions;

    // ============ Events ============

    event ModelRegistered(
        bytes32 indexed modelId,
        address indexed owner,
        string ipfsCid,
        bool isEncrypted
    );

    event AccessGranted(
        bytes32 indexed modelId,
        address indexed user,
        uint8 level,
        uint256 expiresAt
    );

    event AccessRevoked(
        bytes32 indexed modelId,
        address indexed user,
        address indexed revokedBy
    );

    event AccessRequested(
        uint256 indexed requestId,
        bytes32 indexed modelId,
        address indexed requester,
        uint8 level
    );

    event InferenceExecuted(
        bytes32 indexed modelId,
        address indexed user,
        uint256 gasUsed,
        uint256 payment
    );

    event ModelUpdated(
        bytes32 indexed modelId,
        string newIpfsCid,
        uint256 timestamp
    );

    event RevenueWithdrawn(
        address indexed recipient,
        uint256 amount
    );

    // ============ Modifiers ============

    modifier onlyModelOwner(bytes32 modelId) {
        require(models[modelId].owner == msg.sender, "Not model owner");
        _;
    }

    modifier modelExists(bytes32 modelId) {
        require(models[modelId].owner != address(0), "Model not found");
        _;
    }

    modifier hasAccess(bytes32 modelId, uint8 minLevel) {
        AccessGrant memory grant = accessGrants[modelId][msg.sender];
        require(
            msg.sender == models[modelId].owner ||
            (grant.level >= minLevel &&
             !grant.revoked &&
             (grant.expiresAt == 0 || grant.expiresAt > block.timestamp) &&
             (grant.usageLimit == 0 || grant.usageCount < grant.usageLimit)),
            "Insufficient access"
        );
        _;
    }

    // ============ Constructor ============

    constructor() Ownable(msg.sender) {}

    // ============ Model Management ============

    /**
     * @notice Register a new AI model
     * @param modelId Unique identifier for the model
     * @param ipfsCid IPFS content identifier
     * @param isEncrypted Whether the model is encrypted
     * @param accessPrice Price for inference access (in wei)
     */
    function registerModel(
        bytes32 modelId,
        string memory ipfsCid,
        bool isEncrypted,
        uint256 accessPrice
    ) external {
        require(models[modelId].owner == address(0), "Model already exists");

        models[modelId] = ModelInfo({
            owner: msg.sender,
            modelHash: modelId,
            ipfsCid: ipfsCid,
            createdAt: block.timestamp,
            lastUpdated: block.timestamp,
            isEncrypted: isEncrypted,
            accessPrice: accessPrice,
            totalInferences: 0
        });

        // Grant owner full access
        accessGrants[modelId][msg.sender] = AccessGrant({
            level: ACCESS_ADMIN,
            expiresAt: 0, // Never expires
            usageLimit: 0, // Unlimited
            usageCount: 0,
            revoked: false
        });

        emit ModelRegistered(modelId, msg.sender, ipfsCid, isEncrypted);
    }

    /**
     * @notice Update model IPFS CID (for new versions)
     */
    function updateModel(
        bytes32 modelId,
        string memory newIpfsCid
    ) external onlyModelOwner(modelId) {
        models[modelId].ipfsCid = newIpfsCid;
        models[modelId].lastUpdated = block.timestamp;

        emit ModelUpdated(modelId, newIpfsCid, block.timestamp);
    }

    /**
     * @notice Set model metadata
     */
    function setModelMetadata(
        bytes32 modelId,
        string memory category,
        string memory description
    ) external onlyModelOwner(modelId) {
        modelCategories[modelId] = category;
        modelDescriptions[modelId] = description;
    }

    // ============ Access Control ============

    /**
     * @notice Grant access to a user
     */
    function grantAccess(
        bytes32 modelId,
        address user,
        uint8 level,
        uint256 expiresAt,
        uint256 usageLimit
    ) external modelExists(modelId) onlyModelOwner(modelId) {
        require(level > ACCESS_NONE && level <= ACCESS_ADMIN, "Invalid level");
        require(user != address(0), "Invalid user");

        accessGrants[modelId][user] = AccessGrant({
            level: level,
            expiresAt: expiresAt,
            usageLimit: usageLimit,
            usageCount: 0,
            revoked: false
        });

        emit AccessGranted(modelId, user, level, expiresAt);
    }

    /**
     * @notice Revoke access from a user
     */
    function revokeAccess(
        bytes32 modelId,
        address user
    ) external modelExists(modelId) onlyModelOwner(modelId) {
        accessGrants[modelId][user].revoked = true;

        emit AccessRevoked(modelId, user, msg.sender);
    }

    /**
     * @notice Request access to a model
     */
    function requestAccess(
        bytes32 modelId,
        uint8 level,
        string memory reason
    ) external payable modelExists(modelId) returns (uint256 requestId) {
        require(level > ACCESS_NONE && level <= ACCESS_FULL, "Invalid level");
        require(msg.value >= models[modelId].accessPrice, "Insufficient payment");

        requestId = nextRequestId++;

        accessRequests[requestId] = AccessRequest({
            requester: msg.sender,
            modelId: modelId,
            requestedLevel: level,
            payment: msg.value,
            reason: reason,
            timestamp: block.timestamp,
            approved: false
        });

        emit AccessRequested(requestId, modelId, msg.sender, level);

        return requestId;
    }

    /**
     * @notice Approve an access request
     */
    function approveAccessRequest(
        uint256 requestId,
        uint256 expiresAt,
        uint256 usageLimit
    ) external {
        AccessRequest storage request = accessRequests[requestId];
        require(!request.approved, "Already approved");
        require(models[request.modelId].owner == msg.sender, "Not model owner");

        request.approved = true;

        // Grant access
        accessGrants[request.modelId][request.requester] = AccessGrant({
            level: request.requestedLevel,
            expiresAt: expiresAt,
            usageLimit: usageLimit,
            usageCount: 0,
            revoked: false
        });

        // Record revenue
        modelRevenue[request.modelId] += request.payment;
        pendingWithdrawals[msg.sender] += request.payment;

        emit AccessGranted(
            request.modelId,
            request.requester,
            request.requestedLevel,
            expiresAt
        );
    }

    // ============ Inference Execution ============

    /**
     * @notice Execute inference on a model
     */
    function executeInference(
        bytes32 modelId,
        bytes calldata inputData
    ) external payable
      modelExists(modelId)
      hasAccess(modelId, ACCESS_INFERENCE)
      nonReentrant
      returns (bytes memory output)
    {
        // Check payment if required
        uint256 price = models[modelId].accessPrice;
        if (msg.sender != models[modelId].owner && price > 0) {
            require(msg.value >= price, "Insufficient payment");
            modelRevenue[modelId] += msg.value;
            pendingWithdrawals[models[modelId].owner] += msg.value;
        }

        // Update usage count
        accessGrants[modelId][msg.sender].usageCount++;
        models[modelId].totalInferences++;

        // Call inference precompile
        (bool success, bytes memory result) = MODEL_INFERENCE.call(
            abi.encodePacked(modelId, inputData)
        );
        require(success, "Inference failed");

        emit InferenceExecuted(
            modelId,
            msg.sender,
            gasleft(),
            msg.value
        );

        return result;
    }

    /**
     * @notice Execute encrypted inference
     */
    function executeEncryptedInference(
        bytes32 modelId,
        bytes calldata encryptedInput,
        bytes32 proofCommitment
    ) external payable
      modelExists(modelId)
      hasAccess(modelId, ACCESS_INFERENCE)
      nonReentrant
      returns (bytes memory)
    {
        require(models[modelId].isEncrypted, "Model not encrypted");

        // Call encryption precompile for decryption and inference
        (bool success, bytes memory result) = MODEL_ENCRYPTION.call(
            abi.encodePacked(
                uint8(1), // Decrypt operation
                modelId,
                msg.sender,
                encryptedInput,
                proofCommitment
            )
        );
        require(success, "Encrypted inference failed");

        // Update metrics
        accessGrants[modelId][msg.sender].usageCount++;
        models[modelId].totalInferences++;

        return result;
    }

    // ============ Staking ============

    /**
     * @notice Set staking requirement for a model
     */
    function setStakingRequirement(
        bytes32 modelId,
        uint256 amount
    ) external onlyModelOwner(modelId) {
        stakingRequirements[modelId] = amount;
    }

    /**
     * @notice Stake tokens for model access
     */
    function stakeForAccess(bytes32 modelId) external payable {
        require(msg.value >= stakingRequirements[modelId], "Insufficient stake");
        userStakes[msg.sender][modelId] += msg.value;
    }

    /**
     * @notice Unstake tokens
     */
    function unstake(bytes32 modelId, uint256 amount) external {
        require(userStakes[msg.sender][modelId] >= amount, "Insufficient stake");
        userStakes[msg.sender][modelId] -= amount;
        payable(msg.sender).transfer(amount);
    }

    // ============ Revenue Management ============

    /**
     * @notice Withdraw earned revenue
     */
    function withdrawRevenue() external nonReentrant {
        uint256 amount = pendingWithdrawals[msg.sender];
        require(amount > 0, "No pending withdrawals");

        pendingWithdrawals[msg.sender] = 0;
        payable(msg.sender).transfer(amount);

        emit RevenueWithdrawn(msg.sender, amount);
    }

    /**
     * @notice Get model revenue statistics
     */
    function getModelStats(bytes32 modelId) external view returns (
        uint256 revenue,
        uint256 totalInferences,
        uint256 uniqueUsers
    ) {
        revenue = modelRevenue[modelId];
        totalInferences = models[modelId].totalInferences;
        // Unique users would require additional tracking
        uniqueUsers = 0; // Placeholder
    }

    // ============ View Functions ============

    /**
     * @notice Check if user has access to model
     */
    function hasAccessToModel(
        bytes32 modelId,
        address user
    ) external view returns (bool) {
        AccessGrant memory grant = accessGrants[modelId][user];
        return (
            user == models[modelId].owner ||
            (grant.level > ACCESS_NONE &&
             !grant.revoked &&
             (grant.expiresAt == 0 || grant.expiresAt > block.timestamp) &&
             (grant.usageLimit == 0 || grant.usageCount < grant.usageLimit))
        );
    }

    /**
     * @notice Get user's access level for a model
     */
    function getUserAccessLevel(
        bytes32 modelId,
        address user
    ) external view returns (uint8) {
        if (user == models[modelId].owner) {
            return ACCESS_ADMIN;
        }

        AccessGrant memory grant = accessGrants[modelId][user];
        if (grant.revoked ||
            (grant.expiresAt > 0 && grant.expiresAt <= block.timestamp) ||
            (grant.usageLimit > 0 && grant.usageCount >= grant.usageLimit)) {
            return ACCESS_NONE;
        }

        return grant.level;
    }

    /**
     * @notice Get model details
     */
    function getModel(bytes32 modelId) external view returns (
        address owner,
        string memory ipfsCid,
        bool isEncrypted,
        uint256 accessPrice,
        uint256 totalInferences
    ) {
        ModelInfo memory model = models[modelId];
        return (
            model.owner,
            model.ipfsCid,
            model.isEncrypted,
            model.accessPrice,
            model.totalInferences
        );
    }

    // ============ Emergency Functions ============

    /**
     * @notice Emergency pause (only owner)
     */
    function emergencyWithdraw() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }

    /**
     * @notice Update precompile addresses if needed
     */
    function updatePrecompileAddress(
        uint8 precompileId,
        address newAddress
    ) external onlyOwner {
        // Would need additional state variables to track
        // This is a placeholder for upgradability
    }

    // ============ Receive Ether ============

    receive() external payable {}
    fallback() external payable {}
}