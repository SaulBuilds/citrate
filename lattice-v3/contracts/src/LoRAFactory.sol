// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";
import "./lib/AccessControl.sol";

/**
 * @title LoRAFactory
 * @notice Factory for creating and managing LoRA (Low-Rank Adaptation) fine-tunes
 * @dev Integrates with Lattice LoRA precompile for efficient adaptation
 */
contract LoRAFactory is AccessControl {
    // Lattice precompile address
    address constant LORA_PRECOMPILE = 0x0000000000000000000000000000000000001001;
    
    // Structs
    struct LoRAAdapter {
        bytes32 loraHash;
        bytes32 baseModelHash;
        address creator;
        string name;
        string description;
        string ipfsCID;
        uint256 rank;
        uint256 alpha;
        uint256 dropout;
        uint256 createdAt;
        uint256 trainingCost;
        bool isPublic;
        TrainingConfig config;
    }
    
    struct TrainingConfig {
        uint256 epochs;
        uint256 batchSize;
        uint256 learningRate; // Fixed point (1e18 = 1.0)
        string datasetCID;
        uint256 datasetSize;
        uint256 validationSplit; // Percentage in basis points
    }
    
    struct MergeRequest {
        bytes32 requestHash;
        bytes32[] loraHashes;
        uint256[] weights; // Fixed point weights for each LoRA
        address requester;
        bytes32 resultHash;
        string resultCID;
        uint256 mergeType; // 0: Linear, 1: SVD, 2: Task-Arithmetic
        bool completed;
    }
    
    // State variables
    IModelRegistry public modelRegistry;
    
    mapping(bytes32 => LoRAAdapter) public adapters;
    mapping(bytes32 => MergeRequest) public mergeRequests;
    mapping(address => bytes32[]) public userAdapters;
    mapping(bytes32 => bytes32[]) public modelAdapters; // baseModel => LoRAs
    mapping(bytes32 => mapping(address => bool)) public adapterPermissions;
    
    bytes32[] public allAdapterHashes;
    uint256 public totalAdapters;
    uint256 public trainingFeePerEpoch = 0.01 ether; // 0.01 LATT per epoch
    uint256 public mergeFee = 0.05 ether; // 0.05 LATT per merge
    
    // Events
    event LoRACreated(
        bytes32 indexed loraHash,
        bytes32 indexed baseModelHash,
        address indexed creator,
        string name
    );
    
    event LoRAMerged(
        bytes32 indexed requestHash,
        bytes32[] loraHashes,
        bytes32 resultHash
    );
    
    event TrainingStarted(
        bytes32 indexed loraHash,
        uint256 epochs,
        uint256 cost
    );
    
    event TrainingCompleted(
        bytes32 indexed loraHash,
        string ipfsCID
    );
    
    event PermissionGranted(
        bytes32 indexed loraHash,
        address indexed user
    );
    
    constructor(address _modelRegistry) {
        modelRegistry = IModelRegistry(_modelRegistry);
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(OPERATOR_ROLE, msg.sender);
    }
    
    /**
     * @notice Create a new LoRA adapter
     * @param baseModelHash Hash of the base model
     * @param name Name of the LoRA
     * @param description Description of the adaptation
     * @param rank LoRA rank parameter
     * @param alpha LoRA alpha parameter
     * @param dropout Dropout rate (basis points)
     * @param config Training configuration
     */
    function createLoRA(
        bytes32 baseModelHash,
        string memory name,
        string memory description,
        uint256 rank,
        uint256 alpha,
        uint256 dropout,
        TrainingConfig memory config
    ) external payable returns (bytes32) {
        // Verify base model exists
        (address modelOwner,,,,,,,) = modelRegistry.getModel(baseModelHash);
        require(modelOwner != address(0), "Base model not found");
        
        // Check permissions for private models
        require(
            modelRegistry.hasPermission(baseModelHash, msg.sender),
            "No permission for base model"
        );
        
        // Calculate training cost
        uint256 trainingCost = config.epochs * trainingFeePerEpoch;
        require(msg.value >= trainingCost, "Insufficient training fee");
        
        // Generate LoRA hash
        bytes32 loraHash = keccak256(
            abi.encodePacked(
                msg.sender,
                baseModelHash,
                name,
                block.timestamp,
                totalAdapters
            )
        );
        
        // Store LoRA adapter
        LoRAAdapter storage adapter = adapters[loraHash];
        adapter.loraHash = loraHash;
        adapter.baseModelHash = baseModelHash;
        adapter.creator = msg.sender;
        adapter.name = name;
        adapter.description = description;
        adapter.rank = rank;
        adapter.alpha = alpha;
        adapter.dropout = dropout;
        adapter.createdAt = block.timestamp;
        adapter.trainingCost = trainingCost;
        adapter.isPublic = false;
        adapter.config = config;
        
        // Update mappings
        userAdapters[msg.sender].push(loraHash);
        modelAdapters[baseModelHash].push(loraHash);
        allAdapterHashes.push(loraHash);
        totalAdapters++;
        
        // Start training via precompile
        _startTraining(loraHash, config);
        
        emit LoRACreated(loraHash, baseModelHash, msg.sender, name);
        emit TrainingStarted(loraHash, config.epochs, trainingCost);
        
        return loraHash;
    }
    
    /**
     * @notice Complete LoRA training (called by operator after training)
     * @param loraHash Hash of the LoRA
     * @param ipfsCID IPFS CID of trained weights
     */
    function completeTraining(
        bytes32 loraHash,
        string memory ipfsCID
    ) external onlyRole(OPERATOR_ROLE) {
        LoRAAdapter storage adapter = adapters[loraHash];
        require(adapter.creator != address(0), "LoRA not found");
        require(bytes(adapter.ipfsCID).length == 0, "Already completed");
        
        adapter.ipfsCID = ipfsCID;
        
        emit TrainingCompleted(loraHash, ipfsCID);
    }
    
    /**
     * @notice Merge multiple LoRA adapters
     * @param loraHashes Array of LoRA hashes to merge
     * @param weights Weights for each LoRA (must sum to 1e18)
     * @param mergeType Type of merge (0: Linear, 1: SVD, 2: Task-Arithmetic)
     */
    function mergeLoRAs(
        bytes32[] memory loraHashes,
        uint256[] memory weights,
        uint256 mergeType
    ) external payable returns (bytes32) {
        require(loraHashes.length >= 2, "Need at least 2 LoRAs");
        require(loraHashes.length == weights.length, "Length mismatch");
        require(msg.value >= mergeFee, "Insufficient merge fee");
        require(mergeType <= 2, "Invalid merge type");
        
        // Verify all LoRAs have same base model
        bytes32 baseModel = adapters[loraHashes[0]].baseModelHash;
        uint256 totalWeight = 0;
        
        for (uint i = 0; i < loraHashes.length; i++) {
            LoRAAdapter storage adapter = adapters[loraHashes[i]];
            require(adapter.baseModelHash == baseModel, "Different base models");
            require(
                adapter.isPublic || adapter.creator == msg.sender || 
                adapterPermissions[loraHashes[i]][msg.sender],
                "No permission"
            );
            totalWeight += weights[i];
        }
        
        require(totalWeight == 1e18, "Weights must sum to 1");
        
        // Create merge request
        bytes32 requestHash = keccak256(
            abi.encodePacked(
                msg.sender,
                loraHashes,
                weights,
                block.timestamp
            )
        );
        
        MergeRequest storage request = mergeRequests[requestHash];
        request.requestHash = requestHash;
        request.loraHashes = loraHashes;
        request.weights = weights;
        request.requester = msg.sender;
        request.mergeType = mergeType;
        request.completed = false;
        
        // Execute merge via precompile
        _executeMerge(requestHash, loraHashes, weights, mergeType);
        
        return requestHash;
    }
    
    /**
     * @notice Complete merge request (called by operator)
     * @param requestHash Hash of the merge request
     * @param resultCID IPFS CID of merged LoRA
     */
    function completeMerge(
        bytes32 requestHash,
        string memory resultCID
    ) external onlyRole(OPERATOR_ROLE) {
        MergeRequest storage request = mergeRequests[requestHash];
        require(request.requester != address(0), "Request not found");
        require(!request.completed, "Already completed");
        
        // Generate result hash
        bytes32 resultHash = keccak256(abi.encodePacked(requestHash, resultCID));
        
        request.resultHash = resultHash;
        request.resultCID = resultCID;
        request.completed = true;
        
        // Create new LoRA entry for merged result
        LoRAAdapter storage merged = adapters[resultHash];
        merged.loraHash = resultHash;
        merged.baseModelHash = adapters[request.loraHashes[0]].baseModelHash;
        merged.creator = request.requester;
        merged.name = "Merged LoRA";
        merged.description = "Merged from multiple LoRAs";
        merged.ipfsCID = resultCID;
        merged.createdAt = block.timestamp;
        merged.isPublic = false;
        
        userAdapters[request.requester].push(resultHash);
        modelAdapters[merged.baseModelHash].push(resultHash);
        allAdapterHashes.push(resultHash);
        totalAdapters++;
        
        emit LoRAMerged(requestHash, request.loraHashes, resultHash);
    }
    
    /**
     * @notice Apply LoRA to base model for inference
     * @param baseModelHash Hash of base model
     * @param loraHash Hash of LoRA adapter
     * @param inputData Input data for inference
     */
    function inferWithLoRA(
        bytes32 baseModelHash,
        bytes32 loraHash,
        bytes calldata inputData
    ) external payable returns (bytes memory) {
        LoRAAdapter storage adapter = adapters[loraHash];
        require(adapter.baseModelHash == baseModelHash, "LoRA not for this model");
        require(
            adapter.isPublic || adapter.creator == msg.sender || 
            adapterPermissions[loraHash][msg.sender],
            "No permission"
        );
        
        // Get inference price from base model
        (,,,,,uint256 inferencePrice,,) = modelRegistry.getModel(baseModelHash);
        require(msg.value >= inferencePrice, "Insufficient payment");
        
        // Apply LoRA and execute inference via precompile
        bytes memory result = _applyLoRAAndInfer(baseModelHash, loraHash, inputData);
        
        // Distribute payment (80% to base model owner, 20% to LoRA creator)
        if (inferencePrice > 0) {
            uint256 loraShare = (inferencePrice * 20) / 100;
            uint256 modelShare = inferencePrice - loraShare;
            
            (bool success1, ) = adapter.creator.call{value: loraShare}("");
            require(success1, "LoRA payment failed");
            
            // Remaining goes through model registry
            modelRegistry.requestInference{value: modelShare}(baseModelHash, inputData);
        }
        
        return result;
    }
    
    /**
     * @notice Set LoRA as public/private
     * @param loraHash Hash of the LoRA
     * @param isPublic Whether LoRA should be public
     */
    function setPublicStatus(bytes32 loraHash, bool isPublic) external {
        LoRAAdapter storage adapter = adapters[loraHash];
        require(adapter.creator == msg.sender, "Not creator");
        
        adapter.isPublic = isPublic;
    }
    
    /**
     * @notice Grant permission to use LoRA
     * @param loraHash Hash of the LoRA
     * @param user Address to grant permission
     */
    function grantPermission(bytes32 loraHash, address user) external {
        LoRAAdapter storage adapter = adapters[loraHash];
        require(adapter.creator == msg.sender, "Not creator");
        
        adapterPermissions[loraHash][user] = true;
        emit PermissionGranted(loraHash, user);
    }
    
    /**
     * @notice Revoke permission to use LoRA
     * @param loraHash Hash of the LoRA
     * @param user Address to revoke permission
     */
    function revokePermission(bytes32 loraHash, address user) external {
        LoRAAdapter storage adapter = adapters[loraHash];
        require(adapter.creator == msg.sender, "Not creator");
        
        adapterPermissions[loraHash][user] = false;
    }
    
    // View functions
    
    function getLoRA(bytes32 loraHash) external view returns (
        bytes32 baseModelHash,
        address creator,
        string memory name,
        string memory ipfsCID,
        uint256 rank,
        bool isPublic
    ) {
        LoRAAdapter storage adapter = adapters[loraHash];
        return (
            adapter.baseModelHash,
            adapter.creator,
            adapter.name,
            adapter.ipfsCID,
            adapter.rank,
            adapter.isPublic
        );
    }
    
    function getUserLoRAs(address user) external view returns (bytes32[] memory) {
        return userAdapters[user];
    }
    
    function getModelLoRAs(bytes32 modelHash) external view returns (bytes32[] memory) {
        return modelAdapters[modelHash];
    }
    
    function getMergeRequest(bytes32 requestHash) external view returns (
        bytes32[] memory loraHashes,
        uint256[] memory weights,
        address requester,
        bool completed,
        string memory resultCID
    ) {
        MergeRequest storage request = mergeRequests[requestHash];
        return (
            request.loraHashes,
            request.weights,
            request.requester,
            request.completed,
            request.resultCID
        );
    }
    
    // Internal precompile interactions
    
    function _startTraining(bytes32 loraHash, TrainingConfig memory config) internal {
        (bool success, ) = LORA_PRECOMPILE.call(
            abi.encodeWithSignature(
                "startTraining(bytes32,uint256,uint256,uint256,string)",
                loraHash,
                config.epochs,
                config.batchSize,
                config.learningRate,
                config.datasetCID
            )
        );
        require(success, "Training start failed");
    }
    
    function _executeMerge(
        bytes32 requestHash,
        bytes32[] memory loraHashes,
        uint256[] memory weights,
        uint256 mergeType
    ) internal {
        (bool success, ) = LORA_PRECOMPILE.call(
            abi.encodeWithSignature(
                "mergeLoras(bytes32,bytes32[],uint256[],uint256)",
                requestHash,
                loraHashes,
                weights,
                mergeType
            )
        );
        require(success, "Merge execution failed");
    }
    
    function _applyLoRAAndInfer(
        bytes32 baseModelHash,
        bytes32 loraHash,
        bytes calldata inputData
    ) internal returns (bytes memory) {
        (bool success, bytes memory result) = LORA_PRECOMPILE.call(
            abi.encodeWithSignature(
                "applyAndInfer(bytes32,bytes32,bytes)",
                baseModelHash,
                loraHash,
                inputData
            )
        );
        require(success, "LoRA inference failed");
        return result;
    }
    
    // Admin functions
    
    function setTrainingFee(uint256 newFee) external onlyRole(DEFAULT_ADMIN_ROLE) {
        trainingFeePerEpoch = newFee;
    }
    
    function setMergeFee(uint256 newFee) external onlyRole(DEFAULT_ADMIN_ROLE) {
        mergeFee = newFee;
    }
    
    function withdrawFees() external onlyRole(DEFAULT_ADMIN_ROLE) {
        uint256 balance = address(this).balance;
        require(balance > 0, "No fees");
        
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdrawal failed");
    }
}