// SPDX-License-Identifier: MIT

// citrate-v3/contracts/src/ModelMarketplace.sol
pragma solidity ^0.8.24;

import "./interfaces/IModelRegistry.sol";
import "./interfaces/IModelMarketplace.sol";
import "./lib/AccessControl.sol";
import "./lib/ReentrancyGuard.sol";

/**
 * @title ModelMarketplace
 * @notice Decentralized marketplace for AI models on Citrate blockchain
 * @dev Integrates with ModelRegistry for model management and provides marketplace functionality
 */
contract ModelMarketplace is IModelMarketplace, AccessControl, ReentrancyGuard {

    // Constants
    uint256 public constant MARKETPLACE_FEE_BASIS_POINTS = 250; // 2.5%
    uint256 public constant MIN_PRICE = 0.001 ether; // Minimum price for model access
    uint256 public constant MAX_PRICE = 1000 ether; // Maximum price for model access
    uint256 public constant FEATURED_FEE = 1 ether; // Fee to feature a model
    uint256 public constant BASIS_POINTS_DENOMINATOR = 10000;

    // State variables
    IModelRegistry public immutable modelRegistry;
    address public treasuryAddress;
    uint256 public totalListings;
    uint256 public totalSales;
    uint256 public totalVolume;

    // Mappings
    mapping(bytes32 => IModelMarketplace.ModelListing) public listings;
    mapping(uint8 => bytes32[]) public modelsByCategory;
    mapping(address => bytes32[]) public listingsByOwner;
    mapping(bytes32 => IModelMarketplace.Purchase[]) public purchaseHistory;
    mapping(bytes32 => IModelMarketplace.Review[]) public modelReviews;
    mapping(address => mapping(bytes32 => bool)) public hasReviewed;
    mapping(address => mapping(bytes32 => uint256)) public userPurchases; // Track user purchases for verification
    mapping(uint8 => uint256) public categoryCount;

    // Arrays for enumeration
    bytes32[] public allListings;
    bytes32[] public featuredListings;

    // Events are imported from interface

    // Modifiers
    modifier onlyModelOwner(bytes32 modelId) {
        require(listings[modelId].owner == msg.sender, "Not model owner");
        _;
    }

    modifier validPrice(uint256 price) {
        require(price >= MIN_PRICE && price <= MAX_PRICE, "Invalid price");
        _;
    }

    modifier validCategory(uint8 category) {
        require(category <= 10, "Invalid category"); // 0-10 categories
        _;
    }

    modifier modelExists(bytes32 modelId) {
        require(listings[modelId].owner != address(0), "Model not listed");
        _;
    }

    modifier modelActive(bytes32 modelId) {
        require(listings[modelId].active, "Model listing inactive");
        _;
    }

    constructor(address _modelRegistry, address _treasuryAddress) {
        require(_modelRegistry != address(0), "Invalid model registry");
        require(_treasuryAddress != address(0), "Invalid treasury address");

        modelRegistry = IModelRegistry(_modelRegistry);
        treasuryAddress = _treasuryAddress;

        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    /**
     * @notice List a model in the marketplace
     * @param modelId The model hash from ModelRegistry
     * @param basePrice Base price per inference in wei
     * @param discountPrice Discounted price for bulk purchases
     * @param minimumBulkSize Minimum number of inferences for bulk discount
     * @param category Model category (0-10)
     * @param metadataURI IPFS URI for additional marketplace metadata
     */
    function listModel(
        bytes32 modelId,
        uint256 basePrice,
        uint256 discountPrice,
        uint256 minimumBulkSize,
        uint8 category,
        string memory metadataURI
    )
        external
        validPrice(basePrice)
        validCategory(category)
        nonReentrant
    {
        require(discountPrice <= basePrice, "Discount price cannot exceed base price");
        require(minimumBulkSize >= 1, "Minimum bulk size must be at least 1");
        require(bytes(metadataURI).length > 0, "Metadata URI required");

        // Verify the model exists in the registry and caller is the owner
        (
            address owner,
            ,
            ,
            ,
            ,
            ,
            ,
            bool isActive
        ) = modelRegistry.getModel(modelId);

        require(owner != address(0), "Model not found in registry");
        require(owner == msg.sender, "Not model owner");
        require(isActive, "Model is not active in registry");

        // Check if already listed
        if (listings[modelId].owner != address(0)) {
            require(listings[modelId].owner == msg.sender, "Model already listed by another owner");
            // Update existing listing
            _updateListing(modelId, basePrice, discountPrice, minimumBulkSize, category, metadataURI);
        } else {
            // Create new listing
            _createListing(modelId, basePrice, discountPrice, minimumBulkSize, category, metadataURI);
        }
    }

    /**
     * @notice Purchase access to a model
     * @param modelId The model to purchase access to
     * @param quantity Number of inferences to purchase
     */
    function purchaseAccess(bytes32 modelId, uint256 quantity)
        external
        payable
        modelExists(modelId)
        modelActive(modelId)
        nonReentrant
    {
        require(quantity > 0, "Quantity must be greater than 0");

        IModelMarketplace.ModelListing storage listing = listings[modelId];
        bool bulkDiscount = quantity >= listing.minimumBulkSize;
        uint256 pricePerInference = bulkDiscount ? listing.discountPrice : listing.basePrice;
        uint256 totalPrice = pricePerInference * quantity;

        require(msg.value >= totalPrice, "Insufficient payment");

        // Calculate fees
        uint256 marketplaceFee = (totalPrice * MARKETPLACE_FEE_BASIS_POINTS) / BASIS_POINTS_DENOMINATOR;
        uint256 sellerAmount = totalPrice - marketplaceFee;

        // Update listing statistics
        listing.totalSales += quantity;
        listing.totalRevenue += totalPrice;
        listing.lastSaleAt = block.timestamp;

        // Update global statistics
        totalSales += quantity;
        totalVolume += totalPrice;

        // Record purchase
        purchaseHistory[modelId].push(IModelMarketplace.Purchase({
            modelId: modelId,
            buyer: msg.sender,
            price: pricePerInference,
            quantity: quantity,
            timestamp: block.timestamp,
            bulkDiscount: bulkDiscount
        }));

        // Track user purchases for review verification
        userPurchases[msg.sender][modelId] += quantity;

        // Transfer payments
        payable(listing.owner).transfer(sellerAmount);
        payable(treasuryAddress).transfer(marketplaceFee);

        // Refund excess payment
        if (msg.value > totalPrice) {
            payable(msg.sender).transfer(msg.value - totalPrice);
        }

        emit ModelPurchased(modelId, msg.sender, pricePerInference, quantity, bulkDiscount);
    }

    /**
     * @notice Update pricing for a listed model
     * @param modelId The model to update
     * @param newBasePrice New base price per inference
     * @param newDiscountPrice New discount price for bulk purchases
     * @param newMinimumBulkSize New minimum bulk size for discount
     */
    function updatePricing(
        bytes32 modelId,
        uint256 newBasePrice,
        uint256 newDiscountPrice,
        uint256 newMinimumBulkSize
    )
        external
        onlyModelOwner(modelId)
        validPrice(newBasePrice)
    {
        require(newDiscountPrice <= newBasePrice, "Discount price cannot exceed base price");
        require(newMinimumBulkSize >= 1, "Minimum bulk size must be at least 1");

        IModelMarketplace.ModelListing storage listing = listings[modelId];
        uint256 oldBasePrice = listing.basePrice;
        uint256 oldDiscountPrice = listing.discountPrice;

        listing.basePrice = newBasePrice;
        listing.discountPrice = newDiscountPrice;
        listing.minimumBulkSize = newMinimumBulkSize;

        emit PriceUpdated(modelId, oldBasePrice, newBasePrice, oldDiscountPrice, newDiscountPrice);
    }

    /**
     * @notice Update model category
     * @param modelId The model to update
     * @param newCategory New category (0-10)
     */
    function updateCategory(bytes32 modelId, uint8 newCategory)
        external
        onlyModelOwner(modelId)
        validCategory(newCategory)
    {
        IModelMarketplace.ModelListing storage listing = listings[modelId];
        uint8 oldCategory = listing.category;

        // Remove from old category
        _removeFromCategory(modelId, oldCategory);

        // Add to new category
        listing.category = newCategory;
        modelsByCategory[newCategory].push(modelId);
        categoryCount[newCategory]++;
        categoryCount[oldCategory]--;

        emit CategoryUpdated(modelId, oldCategory, newCategory);
    }

    /**
     * @notice Feature a model (admin or paid by owner)
     * @param modelId The model to feature
     */
    function featureModel(bytes32 modelId)
        external
        payable
        modelExists(modelId)
    {
        IModelMarketplace.ModelListing storage listing = listings[modelId];

        if (hasRole(DEFAULT_ADMIN_ROLE, msg.sender)) {
            // Admin can feature for free
            require(msg.value == 0, "Admin should not send payment");
        } else {
            // Model owner can pay to feature
            require(msg.sender == listing.owner, "Only owner can pay to feature model");
            require(msg.value >= FEATURED_FEE, "Insufficient fee for featuring");

            payable(treasuryAddress).transfer(msg.value);
        }

        if (!listing.featured) {
            listing.featured = true;
            featuredListings.push(modelId);

            emit ModelFeatured(modelId, listing.owner);
        }
    }

    /**
     * @notice Remove featured status from a model
     * @param modelId The model to unfeature
     */
    function unfeatureModel(bytes32 modelId)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        modelExists(modelId)
    {
        IModelMarketplace.ModelListing storage listing = listings[modelId];

        if (listing.featured) {
            listing.featured = false;
            _removeFromFeatured(modelId);

            emit ModelUnfeatured(modelId);
        }
    }

    /**
     * @notice Add a review for a model
     * @param modelId The model to review
     * @param rating Rating from 1-5 stars
     * @param comment Review comment
     */
    function addReview(bytes32 modelId, uint8 rating, string memory comment)
        external
        modelExists(modelId)
    {
        require(rating >= 1 && rating <= 5, "Rating must be between 1 and 5");
        require(!hasReviewed[msg.sender][modelId], "Already reviewed this model");
        require(bytes(comment).length <= 500, "Comment too long");

        bool verified = userPurchases[msg.sender][modelId] > 0;

        modelReviews[modelId].push(IModelMarketplace.Review({
            reviewer: msg.sender,
            modelId: modelId,
            rating: rating,
            comment: comment,
            timestamp: block.timestamp,
            verified: verified
        }));

        hasReviewed[msg.sender][modelId] = true;

        // Update rating statistics
        IModelMarketplace.ModelListing storage listing = listings[modelId];
        listing.totalRating += rating;
        listing.reviewCount++;
        listing.averageRating = (listing.totalRating * 100) / listing.reviewCount; // Store as percentage

        emit ReviewAdded(modelId, msg.sender, rating, verified);
    }

    /**
     * @notice Deactivate a model listing
     * @param modelId The model to deactivate
     */
    function deactivateListing(bytes32 modelId)
        external
        onlyModelOwner(modelId)
    {
        listings[modelId].active = false;
        emit ListingDeactivated(modelId);
    }

    /**
     * @notice Activate a model listing
     * @param modelId The model to activate
     */
    function activateListing(bytes32 modelId)
        external
        onlyModelOwner(modelId)
    {
        listings[modelId].active = true;
        emit ListingActivated(modelId);
    }

    // View functions

    /**
     * @notice Get model listing details
     * @param modelId The model to query
     * @return The model listing data
     */
    function getListing(bytes32 modelId)
        external
        view
        returns (IModelMarketplace.ModelListing memory)
    {
        return listings[modelId];
    }

    /**
     * @notice Search models by category
     * @param category The category to search (0-10)
     * @return Array of model IDs in the category
     */
    function getModelsByCategory(uint8 category)
        external
        view
        validCategory(category)
        returns (bytes32[] memory)
    {
        return modelsByCategory[category];
    }

    /**
     * @notice Get featured models
     * @return Array of featured model IDs
     */
    function getFeaturedModels()
        external
        view
        returns (bytes32[] memory)
    {
        return featuredListings;
    }

    /**
     * @notice Get top rated models
     * @param limit Maximum number of models to return
     * @return Array of top rated model IDs
     */
    function getTopRatedModels(uint256 limit)
        external
        view
        returns (bytes32[] memory)
    {
        bytes32[] memory allModels = allListings;
        uint256 resultCount = limit > allModels.length ? allModels.length : limit;
        bytes32[] memory result = new bytes32[](resultCount);

        // Simple sorting by average rating (could be optimized with a heap)
        uint256[] memory ratings = new uint256[](allModels.length);
        for (uint256 i = 0; i < allModels.length; i++) {
            ratings[i] = listings[allModels[i]].averageRating;
        }

        // Sort and take top N
        for (uint256 i = 0; i < resultCount; i++) {
            uint256 maxIndex = 0;
            uint256 maxRating = 0;

            for (uint256 j = 0; j < allModels.length; j++) {
                if (ratings[j] > maxRating) {
                    maxRating = ratings[j];
                    maxIndex = j;
                }
            }

            if (maxRating > 0) {
                result[i] = allModels[maxIndex];
                ratings[maxIndex] = 0; // Mark as used
            }
        }

        return result;
    }

    /**
     * @notice Get models by owner
     * @param owner The owner address
     * @return Array of model IDs owned by the address
     */
    function getModelsByOwner(address owner)
        external
        view
        returns (bytes32[] memory)
    {
        return listingsByOwner[owner];
    }

    /**
     * @notice Get purchase history for a model
     * @param modelId The model to query
     * @return Array of purchases for the model
     */
    function getPurchaseHistory(bytes32 modelId)
        external
        view
        returns (IModelMarketplace.Purchase[] memory)
    {
        return purchaseHistory[modelId];
    }

    /**
     * @notice Get reviews for a model
     * @param modelId The model to query
     * @return Array of reviews for the model
     */
    function getModelReviews(bytes32 modelId)
        external
        view
        returns (IModelMarketplace.Review[] memory)
    {
        return modelReviews[modelId];
    }

    /**
     * @notice Get marketplace statistics
     * @return totalListings Total number of listings
     * @return totalSales Total number of sales
     * @return totalVolume Total volume traded
     */
    function getMarketplaceStats()
        external
        view
        returns (uint256, uint256, uint256)
    {
        return (totalListings, totalSales, totalVolume);
    }

    // Admin functions

    /**
     * @notice Update treasury address (admin only)
     * @param newTreasury The new treasury address
     */
    function updateTreasuryAddress(address newTreasury)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        require(newTreasury != address(0), "Invalid treasury address");
        treasuryAddress = newTreasury;
    }

    // Internal functions

    function _createListing(
        bytes32 modelId,
        uint256 basePrice,
        uint256 discountPrice,
        uint256 minimumBulkSize,
        uint8 category,
        string memory metadataURI
    ) internal {
        listings[modelId] = IModelMarketplace.ModelListing({
            modelId: modelId,
            owner: msg.sender,
            basePrice: basePrice,
            discountPrice: discountPrice,
            minimumBulkSize: minimumBulkSize,
            totalSales: 0,
            totalRevenue: 0,
            category: category,
            metadataURI: metadataURI,
            featured: false,
            active: true,
            listedAt: block.timestamp,
            lastSaleAt: 0,
            totalRating: 0,
            reviewCount: 0,
            averageRating: 0
        });

        allListings.push(modelId);
        listingsByOwner[msg.sender].push(modelId);
        modelsByCategory[category].push(modelId);
        categoryCount[category]++;
        totalListings++;

        emit ModelListed(modelId, msg.sender, basePrice, discountPrice, category);
    }

    function _updateListing(
        bytes32 modelId,
        uint256 basePrice,
        uint256 discountPrice,
        uint256 minimumBulkSize,
        uint8 category,
        string memory metadataURI
    ) internal {
        IModelMarketplace.ModelListing storage listing = listings[modelId];
        uint8 oldCategory = listing.category;

        listing.basePrice = basePrice;
        listing.discountPrice = discountPrice;
        listing.minimumBulkSize = minimumBulkSize;
        listing.metadataURI = metadataURI;

        if (oldCategory != category) {
            _removeFromCategory(modelId, oldCategory);
            listing.category = category;
            modelsByCategory[category].push(modelId);
            categoryCount[category]++;
            categoryCount[oldCategory]--;
        }

        emit ModelListed(modelId, msg.sender, basePrice, discountPrice, category);
    }

    function _removeFromCategory(bytes32 modelId, uint8 category) internal {
        bytes32[] storage categoryModels = modelsByCategory[category];
        for (uint256 i = 0; i < categoryModels.length; i++) {
            if (categoryModels[i] == modelId) {
                categoryModels[i] = categoryModels[categoryModels.length - 1];
                categoryModels.pop();
                break;
            }
        }
    }

    function _removeFromFeatured(bytes32 modelId) internal {
        for (uint256 i = 0; i < featuredListings.length; i++) {
            if (featuredListings[i] == modelId) {
                featuredListings[i] = featuredListings[featuredListings.length - 1];
                featuredListings.pop();
                break;
            }
        }
    }
}