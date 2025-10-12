// SPDX-License-Identifier: MIT

// lattice-v3/contracts/src/InferenceRouter.sol
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";
import "./lib/AccessControl.sol";

/**
 * @title InferenceRouter
 * @notice Routes inference requests to appropriate models and compute providers
 * @dev Manages load balancing, caching, and payment distribution
 */
contract InferenceRouter is AccessControl {
    // Structs
    struct InferenceRequest {
        uint256 requestId;
        address requester;
        bytes32 modelHash;
        bytes inputData;
        uint256 maxPrice;
        uint256 timestamp;
        RequestStatus status;
        bytes outputData;
        address computeProvider;
        uint256 pricePaid;
    }
    
    struct ComputeProvider {
        address provider;
        string endpoint;
        uint256 stake;
        uint256 minPrice;
        uint256 maxConcurrent;
        uint256 currentLoad;
        uint256 totalInferences;
        uint256 successRate; // Basis points (10000 = 100%)
        bool isActive;
        bytes32[] supportedModels;
    }
    
    struct ModelRoute {
        bytes32 modelHash;
        address[] providers;
        uint256 avgResponseTime;
        uint256 totalRequests;
        bool cachingEnabled;
    }
    
    enum RequestStatus {
        Pending,
        Processing,
        Completed,
        Failed,
        Cancelled
    }
    
    // State variables
    IModelRegistry public modelRegistry;
    
    mapping(uint256 => InferenceRequest) public requests;
    mapping(address => ComputeProvider) public providers;
    mapping(bytes32 => ModelRoute) public routes;
    mapping(bytes32 => mapping(bytes32 => bytes)) public responseCache; // modelHash => inputHash => output
    mapping(address => uint256[]) public userRequests;
    mapping(address => uint256) public providerBalances;
    
    address[] public allProviders;
    uint256 public nextRequestId;
    uint256 public minProviderStake = 100 ether; // 100 LATT
    uint256 public platformFee = 250; // 2.5% in basis points
    uint256 public cacheReward = 100; // 1% reward for cache hits
    
    // Events
    event InferenceRequested(
        uint256 indexed requestId,
        address indexed requester,
        bytes32 indexed modelHash
    );
    
    event InferenceCompleted(
        uint256 indexed requestId,
        address indexed provider,
        uint256 price
    );
    
    event ProviderRegistered(
        address indexed provider,
        uint256 stake
    );
    
    event ProviderUpdated(
        address indexed provider,
        bool isActive
    );
    
    event RouteUpdated(
        bytes32 indexed modelHash,
        address[] providers
    );
    
    event CacheHit(
        bytes32 indexed modelHash,
        bytes32 inputHash
    );
    
    constructor(address _modelRegistry) {
        modelRegistry = IModelRegistry(_modelRegistry);
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(OPERATOR_ROLE, msg.sender);
    }
    
    /**
     * @notice Register as a compute provider
     * @param endpoint API endpoint for inference
     * @param minPrice Minimum price per inference
     * @param supportedModels Array of model hashes this provider supports
     */
    function registerProvider(
        string memory endpoint,
        uint256 minPrice,
        bytes32[] memory supportedModels
    ) external payable {
        require(msg.value >= minProviderStake, "Insufficient stake");
        require(bytes(endpoint).length > 0, "Endpoint required");
        require(supportedModels.length > 0, "Must support at least one model");
        
        ComputeProvider storage provider = providers[msg.sender];
        require(!provider.isActive, "Provider already registered");
        
        provider.provider = msg.sender;
        provider.endpoint = endpoint;
        provider.stake = msg.value;
        provider.minPrice = minPrice;
        provider.maxConcurrent = 10; // Default
        provider.successRate = 10000; // Start at 100%
        provider.isActive = true;
        provider.supportedModels = supportedModels;
        
        allProviders.push(msg.sender);
        
        // Update routes for supported models
        for (uint i = 0; i < supportedModels.length; i++) {
            routes[supportedModels[i]].providers.push(msg.sender);
        }
        
        emit ProviderRegistered(msg.sender, msg.value);
    }
    
    /**
     * @notice Request inference from a model
     * @param modelHash Hash of the model to use
     * @param inputData Input data for inference
     * @param maxPrice Maximum price willing to pay
     */
    function requestInference(
        bytes32 modelHash,
        bytes calldata inputData,
        uint256 maxPrice
    ) external payable returns (uint256) {
        require(msg.value >= maxPrice, "Insufficient payment");
        
        // Check cache first
        bytes32 inputHash = keccak256(inputData);
        bytes memory cachedResult = responseCache[modelHash][inputHash];
        
        if (cachedResult.length > 0 && routes[modelHash].cachingEnabled) {
            // Return cached result
            emit CacheHit(modelHash, inputHash);
            
            // Refund most of the payment (keep small cache reward)
            uint256 cacheRewardAmount = (maxPrice * cacheReward) / 10000;
            if (msg.value > cacheRewardAmount) {
                (bool success, ) = msg.sender.call{value: msg.value - cacheRewardAmount}("");
                require(success, "Refund failed");
            }
            
            // Create completed request record
            uint256 cachedRequestId = nextRequestId++;
            InferenceRequest storage cachedRequest = requests[cachedRequestId];
            cachedRequest.requestId = cachedRequestId;
            cachedRequest.requester = msg.sender;
            cachedRequest.modelHash = modelHash;
            cachedRequest.inputData = inputData;
            cachedRequest.maxPrice = maxPrice;
            cachedRequest.timestamp = block.timestamp;
            cachedRequest.status = RequestStatus.Completed;
            cachedRequest.outputData = cachedResult;
            cachedRequest.pricePaid = cacheRewardAmount;
            
            userRequests[msg.sender].push(cachedRequestId);
            
            return cachedRequestId;
        }
        
        // Create new inference request
        uint256 requestId = nextRequestId++;
        InferenceRequest storage request = requests[requestId];
        request.requestId = requestId;
        request.requester = msg.sender;
        request.modelHash = modelHash;
        request.inputData = inputData;
        request.maxPrice = maxPrice;
        request.timestamp = block.timestamp;
        request.status = RequestStatus.Pending;
        
        userRequests[msg.sender].push(requestId);
        
        // Find best provider
        address bestProvider = _selectProvider(modelHash, maxPrice);
        require(bestProvider != address(0), "No available provider");
        
        request.computeProvider = bestProvider;
        request.status = RequestStatus.Processing;
        
        // Update provider load
        providers[bestProvider].currentLoad++;
        
        emit InferenceRequested(requestId, msg.sender, modelHash);
        
        return requestId;
    }
    
    /**
     * @notice Complete an inference request (called by provider)
     * @param requestId ID of the request
     * @param outputData Output from the model
     */
    function completeInference(
        uint256 requestId,
        bytes calldata outputData
    ) external {
        InferenceRequest storage request = requests[requestId];
        require(request.computeProvider == msg.sender, "Not assigned provider");
        require(request.status == RequestStatus.Processing, "Invalid status");
        
        request.outputData = outputData;
        request.status = RequestStatus.Completed;
        
        // Calculate payment
        ComputeProvider storage provider = providers[msg.sender];
        uint256 price = provider.minPrice;
        if (price > request.maxPrice) {
            price = request.maxPrice;
        }
        
        // Apply platform fee
        uint256 platformAmount = (price * platformFee) / 10000;
        uint256 providerAmount = price - platformAmount;
        
        request.pricePaid = price;
        providerBalances[msg.sender] += providerAmount;
        
        // Update provider stats
        provider.currentLoad--;
        provider.totalInferences++;
        
        // Update route stats
        routes[request.modelHash].totalRequests++;
        
        // Cache result if enabled
        if (routes[request.modelHash].cachingEnabled) {
            bytes32 inputHash = keccak256(request.inputData);
            responseCache[request.modelHash][inputHash] = outputData;
        }
        
        // Refund excess payment
        if (request.maxPrice > price) {
            (bool success, ) = request.requester.call{value: request.maxPrice - price}("");
            require(success, "Refund failed");
        }
        
        emit InferenceCompleted(requestId, msg.sender, price);
    }
    
    /**
     * @notice Cancel an inference request
     * @param requestId ID of the request
     */
    function cancelRequest(uint256 requestId) external {
        InferenceRequest storage request = requests[requestId];
        require(request.requester == msg.sender, "Not request owner");
        require(request.status == RequestStatus.Pending, "Cannot cancel");
        
        request.status = RequestStatus.Cancelled;
        
        // Refund payment
        (bool success, ) = msg.sender.call{value: request.maxPrice}("");
        require(success, "Refund failed");
    }
    
    /**
     * @notice Withdraw earnings as a provider
     */
    function withdrawEarnings() external {
        uint256 balance = providerBalances[msg.sender];
        require(balance > 0, "No earnings");
        
        providerBalances[msg.sender] = 0;
        
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdrawal failed");
    }
    
    /**
     * @notice Update provider status
     * @param isActive Whether provider is active
     */
    function updateProviderStatus(bool isActive) external {
        ComputeProvider storage provider = providers[msg.sender];
        require(provider.stake > 0, "Not registered");
        
        provider.isActive = isActive;
        emit ProviderUpdated(msg.sender, isActive);
    }
    
    /**
     * @notice Add stake as a provider
     */
    function addStake() external payable {
        ComputeProvider storage provider = providers[msg.sender];
        require(provider.stake > 0, "Not registered");
        
        provider.stake += msg.value;
    }
    
    /**
     * @notice Withdraw stake (only if not active)
     * @param amount Amount to withdraw
     */
    function withdrawStake(uint256 amount) external {
        ComputeProvider storage provider = providers[msg.sender];
        require(!provider.isActive, "Must deactivate first");
        require(provider.currentLoad == 0, "Has active requests");
        require(provider.stake >= amount, "Insufficient stake");
        
        provider.stake -= amount;
        
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Withdrawal failed");
    }
    
    /**
     * @notice Enable/disable caching for a model route
     * @param modelHash Hash of the model
     * @param enabled Whether caching is enabled
     */
    function setCaching(bytes32 modelHash, bool enabled) external onlyRole(OPERATOR_ROLE) {
        routes[modelHash].cachingEnabled = enabled;
    }
    
    /**
     * @notice Set platform fee
     * @param newFee New fee in basis points
     */
    function setPlatformFee(uint256 newFee) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newFee <= 1000, "Fee too high"); // Max 10%
        platformFee = newFee;
    }
    
    /**
     * @notice Set minimum provider stake
     * @param newStake New minimum stake amount
     */
    function setMinProviderStake(uint256 newStake) external onlyRole(DEFAULT_ADMIN_ROLE) {
        minProviderStake = newStake;
    }
    
    // View functions
    
    function getRequest(uint256 requestId) external view returns (
        address requester,
        bytes32 modelHash,
        RequestStatus status,
        bytes memory outputData,
        uint256 pricePaid
    ) {
        InferenceRequest storage request = requests[requestId];
        return (
            request.requester,
            request.modelHash,
            request.status,
            request.outputData,
            request.pricePaid
        );
    }
    
    function getUserRequests(address user) external view returns (uint256[] memory) {
        return userRequests[user];
    }
    
    function getProviders(bytes32 modelHash) external view returns (address[] memory) {
        return routes[modelHash].providers;
    }
    
    function getProviderInfo(address provider) external view returns (
        string memory endpoint,
        uint256 stake,
        uint256 currentLoad,
        uint256 totalInferences,
        bool isActive
    ) {
        ComputeProvider storage p = providers[provider];
        return (
            p.endpoint,
            p.stake,
            p.currentLoad,
            p.totalInferences,
            p.isActive
        );
    }
    
    // Internal functions
    
    function _selectProvider(
        bytes32 modelHash,
        uint256 maxPrice
    ) internal view returns (address) {
        address[] memory routeProviders = routes[modelHash].providers;
        address bestProvider = address(0);
        uint256 bestScore = 0;
        
        for (uint i = 0; i < routeProviders.length; i++) {
            ComputeProvider storage provider = providers[routeProviders[i]];
            
            if (!provider.isActive) continue;
            if (provider.minPrice > maxPrice) continue;
            if (provider.currentLoad >= provider.maxConcurrent) continue;
            
            // Score based on: affordability (40%), load (30%), success rate (30%)
            // Lower minPrice should yield higher score. Normalize to [0..10000].
            uint256 priceScore = 10000 - ((10000 * provider.minPrice) / maxPrice);
            uint256 loadScore = 10000 - (10000 * provider.currentLoad / provider.maxConcurrent);
            uint256 score = (priceScore * 40 + loadScore * 30 + provider.successRate * 30) / 100;
            
            if (score > bestScore) {
                bestScore = score;
                bestProvider = routeProviders[i];
            }
        }
        
        return bestProvider;
    }
    
    // Admin functions
    
    function withdrawPlatformFees() external onlyRole(DEFAULT_ADMIN_ROLE) {
        uint256 balance = address(this).balance;
        
        // Subtract provider balances
        for (uint i = 0; i < allProviders.length; i++) {
            balance -= providerBalances[allProviders[i]];
        }
        
        require(balance > 0, "No fees to withdraw");
        
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdrawal failed");
    }
}
