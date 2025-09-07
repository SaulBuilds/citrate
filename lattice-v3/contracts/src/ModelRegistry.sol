// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";
import "./lib/AccessControl.sol";

/**
 * @title ModelRegistry
 * @notice Registry for AI models on Lattice blockchain
 * @dev Integrates with Lattice precompiles for model operations
 */
contract ModelRegistry is IModelRegistry, AccessControl {
    // Lattice precompile addresses
    address constant MODEL_PRECOMPILE = 0x0000000000000000000000000000000000001000;
    address constant ARTIFACT_PRECOMPILE = 0x0000000000000000000000000000000000001002;
    
    // Model information
    struct Model {
        bytes32 modelHash;
        address owner;
        string name;
        string framework;
        string version;
        string ipfsCID;  // IPFS content ID for model weights
        uint256 sizeBytes;
        uint256 inferencePrice;  // Price in LATT wei per inference
        uint256 totalInferences;
        uint256 createdAt;
        uint256 updatedAt;
        bool isActive;
        IModelRegistry.ModelMetadata metadata;
    }
    
    // State variables
    mapping(bytes32 => Model) public models;
    mapping(address => bytes32[]) public ownerModels;
    mapping(bytes32 => mapping(address => bool)) public modelPermissions;
    mapping(bytes32 => uint256) public modelRevenue;
    
    bytes32[] public allModelHashes;
    uint256 public totalModels;
    uint256 public constant REGISTRATION_FEE = 0.1 ether; // 0.1 LATT
    
    // Events
    event ModelRegistered(
        bytes32 indexed modelHash,
        address indexed owner,
        string name,
        string ipfsCID
    );
    
    event ModelUpdated(
        bytes32 indexed modelHash,
        string newVersion,
        string newIpfsCID
    );
    
    event ModelDeactivated(bytes32 indexed modelHash);
    event ModelActivated(bytes32 indexed modelHash);
    
    event InferenceRequested(
        bytes32 indexed modelHash,
        address indexed requester,
        uint256 price
    );
    
    event PermissionGranted(
        bytes32 indexed modelHash,
        address indexed user
    );
    
    event PermissionRevoked(
        bytes32 indexed modelHash,
        address indexed user
    );
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(OPERATOR_ROLE, msg.sender);
    }
    
    /**
     * @notice Register a new AI model
     * @param name Model name
     * @param framework ML framework (PyTorch, TensorFlow, etc.)
     * @param version Model version
     * @param ipfsCID IPFS CID for model weights
     * @param sizeBytes Model size in bytes
     * @param inferencePrice Price per inference in LATT wei
     * @param metadata Additional model metadata
     */
    function registerModel(
        string memory name,
        string memory framework,
        string memory version,
        string memory ipfsCID,
        uint256 sizeBytes,
        uint256 inferencePrice,
        IModelRegistry.ModelMetadata memory metadata
    ) external payable returns (bytes32) {
        require(msg.value >= REGISTRATION_FEE, "Insufficient registration fee");
        require(bytes(name).length > 0, "Name required");
        require(bytes(ipfsCID).length > 0, "IPFS CID required");
        
        // Generate model hash
        bytes32 modelHash = keccak256(
            abi.encodePacked(
                msg.sender,
                name,
                block.timestamp,
                totalModels
            )
        );
        
        require(models[modelHash].createdAt == 0, "Model already exists");
        
        // Store model information
        Model storage model = models[modelHash];
        model.modelHash = modelHash;
        model.owner = msg.sender;
        model.name = name;
        model.framework = framework;
        model.version = version;
        model.ipfsCID = ipfsCID;
        model.sizeBytes = sizeBytes;
        model.inferencePrice = inferencePrice;
        model.createdAt = block.timestamp;
        model.updatedAt = block.timestamp;
        model.isActive = true;
        model.metadata = metadata;
        
        // Update mappings
        ownerModels[msg.sender].push(modelHash);
        allModelHashes.push(modelHash);
        totalModels++;
        
        // Call Lattice precompile to register model on-chain
        _registerWithPrecompile(modelHash, ipfsCID);
        
        emit ModelRegistered(modelHash, msg.sender, name, ipfsCID);
        
        return modelHash;
    }
    
    /**
     * @notice Update model version and weights
     * @param modelHash Hash of the model to update
     * @param newVersion New version string
     * @param newIpfsCID New IPFS CID for updated weights
     */
    function updateModel(
        bytes32 modelHash,
        string memory newVersion,
        string memory newIpfsCID
    ) external {
        Model storage model = models[modelHash];
        require(model.owner == msg.sender, "Not model owner");
        require(model.isActive, "Model not active");
        
        model.version = newVersion;
        model.ipfsCID = newIpfsCID;
        model.updatedAt = block.timestamp;
        
        // Update precompile registration
        _registerWithPrecompile(modelHash, newIpfsCID);
        
        emit ModelUpdated(modelHash, newVersion, newIpfsCID);
    }
    
    /**
     * @notice Set inference price for a model
     * @param modelHash Hash of the model
     * @param newPrice New price in LATT wei
     */
    function setInferencePrice(bytes32 modelHash, uint256 newPrice) external {
        Model storage model = models[modelHash];
        require(model.owner == msg.sender, "Not model owner");
        
        model.inferencePrice = newPrice;
        model.updatedAt = block.timestamp;
    }
    
    /**
     * @notice Grant permission to use model
     * @param modelHash Hash of the model
     * @param user Address to grant permission to
     */
    function grantPermission(bytes32 modelHash, address user) external {
        Model storage model = models[modelHash];
        require(model.owner == msg.sender, "Not model owner");
        
        modelPermissions[modelHash][user] = true;
        emit PermissionGranted(modelHash, user);
    }
    
    /**
     * @notice Revoke permission to use model
     * @param modelHash Hash of the model
     * @param user Address to revoke permission from
     */
    function revokePermission(bytes32 modelHash, address user) external {
        Model storage model = models[modelHash];
        require(model.owner == msg.sender, "Not model owner");
        
        modelPermissions[modelHash][user] = false;
        emit PermissionRevoked(modelHash, user);
    }
    
    /**
     * @notice Request inference from a model
     * @param modelHash Hash of the model
     * @param inputData Encoded input data
     */
    function requestInference(
        bytes32 modelHash,
        bytes calldata inputData
    ) external payable returns (bytes memory) {
        Model storage model = models[modelHash];
        require(model.isActive, "Model not active");
        require(msg.value >= model.inferencePrice, "Insufficient payment");
        
        // Check permissions
        if (model.inferencePrice > 0) {
            require(
                model.owner == msg.sender || modelPermissions[modelHash][msg.sender],
                "No permission"
            );
        }
        
        // Update statistics
        model.totalInferences++;
        modelRevenue[modelHash] += msg.value;
        
        // Transfer payment to model owner
        if (msg.value > 0) {
            (bool success, ) = model.owner.call{value: msg.value}("");
            require(success, "Payment failed");
        }
        
        emit InferenceRequested(modelHash, msg.sender, msg.value);
        
        // Call precompile for actual inference
        return _executeInference(modelHash, inputData);
    }
    
    /**
     * @notice Deactivate a model
     * @param modelHash Hash of the model
     */
    function deactivateModel(bytes32 modelHash) external {
        Model storage model = models[modelHash];
        require(
            model.owner == msg.sender || hasRole(OPERATOR_ROLE, msg.sender),
            "Not authorized"
        );
        
        model.isActive = false;
        emit ModelDeactivated(modelHash);
    }
    
    /**
     * @notice Activate a model
     * @param modelHash Hash of the model
     */
    function activateModel(bytes32 modelHash) external {
        Model storage model = models[modelHash];
        require(
            model.owner == msg.sender || hasRole(OPERATOR_ROLE, msg.sender),
            "Not authorized"
        );
        
        model.isActive = true;
        emit ModelActivated(modelHash);
    }
    
    /**
     * @notice Get model details
     * @param modelHash Hash of the model
     */
    function getModel(bytes32 modelHash) external view returns (
        address owner,
        string memory name,
        string memory framework,
        string memory version,
        string memory ipfsCID,
        uint256 inferencePrice,
        uint256 totalInferences,
        bool isActive
    ) {
        Model storage model = models[modelHash];
        return (
            model.owner,
            model.name,
            model.framework,
            model.version,
            model.ipfsCID,
            model.inferencePrice,
            model.totalInferences,
            model.isActive
        );
    }
    
    /**
     * @notice Get models owned by an address
     * @param owner Address of the owner
     */
    function getModelsByOwner(address owner) external view returns (bytes32[] memory) {
        return ownerModels[owner];
    }
    
    /**
     * @notice Get total revenue for a model
     * @param modelHash Hash of the model
     */
    function getModelRevenue(bytes32 modelHash) external view returns (uint256) {
        return modelRevenue[modelHash];
    }
    
    /**
     * @notice Check if user has permission for model
     * @param modelHash Hash of the model
     * @param user Address to check
     */
    function hasPermission(bytes32 modelHash, address user) external view returns (bool) {
        Model storage model = models[modelHash];
        return model.owner == user || modelPermissions[modelHash][user];
    }
    
    // Internal functions for precompile interaction
    
    function _registerWithPrecompile(bytes32 modelHash, string memory ipfsCID) internal {
        // Call Lattice MODEL_PRECOMPILE to register model
        (bool success, ) = MODEL_PRECOMPILE.call(
            abi.encodeWithSignature(
                "registerModel(bytes32,string)",
                modelHash,
                ipfsCID
            )
        );
        require(success, "Precompile registration failed");
    }
    
    function _executeInference(
        bytes32 modelHash,
        bytes calldata inputData
    ) internal returns (bytes memory) {
        // Call Lattice MODEL_PRECOMPILE for inference
        (bool success, bytes memory result) = MODEL_PRECOMPILE.call(
            abi.encodeWithSignature(
                "executeInference(bytes32,bytes)",
                modelHash,
                inputData
            )
        );
        require(success, "Inference execution failed");
        return result;
    }
    
    // Admin functions
    
    function withdrawFees() external onlyRole(DEFAULT_ADMIN_ROLE) {
        uint256 balance = address(this).balance;
        require(balance > 0, "No fees to withdraw");
        
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdrawal failed");
    }
    
    function setRegistrationFee(uint256 newFee) external onlyRole(DEFAULT_ADMIN_ROLE) {
        // Registration fee is constant in this version
        revert("Registration fee is immutable");
    }
}