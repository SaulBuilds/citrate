// SPDX-License-Identifier: MIT

// lattice-v3/contracts/src/interfaces/IModelMarketplace.sol
pragma solidity ^0.8.24;

interface IModelMarketplace {
    struct ModelListing {
        bytes32 modelId;
        address owner;
        uint256 basePrice;
        uint256 discountPrice;
        uint256 minimumBulkSize;
        uint256 totalSales;
        uint256 totalRevenue;
        uint8 category;
        string metadataURI;
        bool featured;
        bool active;
        uint256 listedAt;
        uint256 lastSaleAt;
        uint256 totalRating;
        uint256 reviewCount;
        uint256 averageRating;
    }

    struct Purchase {
        bytes32 modelId;
        address buyer;
        uint256 price;
        uint256 quantity;
        uint256 timestamp;
        bool bulkDiscount;
    }

    struct Review {
        address reviewer;
        bytes32 modelId;
        uint8 rating;
        string comment;
        uint256 timestamp;
        bool verified;
    }

    // Events
    event ModelListed(
        bytes32 indexed modelId,
        address indexed owner,
        uint256 basePrice,
        uint256 discountPrice,
        uint8 category
    );

    event ModelPurchased(
        bytes32 indexed modelId,
        address indexed buyer,
        uint256 price,
        uint256 quantity,
        bool bulkDiscount
    );

    event PriceUpdated(
        bytes32 indexed modelId,
        uint256 oldPrice,
        uint256 newPrice,
        uint256 oldDiscountPrice,
        uint256 newDiscountPrice
    );

    event ModelFeatured(bytes32 indexed modelId, address indexed owner);
    event ModelUnfeatured(bytes32 indexed modelId);

    event ReviewAdded(
        bytes32 indexed modelId,
        address indexed reviewer,
        uint8 rating,
        bool verified
    );

    event CategoryUpdated(bytes32 indexed modelId, uint8 oldCategory, uint8 newCategory);
    event ListingDeactivated(bytes32 indexed modelId);
    event ListingActivated(bytes32 indexed modelId);

    // Core marketplace functions
    function listModel(
        bytes32 modelId,
        uint256 basePrice,
        uint256 discountPrice,
        uint256 minimumBulkSize,
        uint8 category,
        string memory metadataURI
    ) external;

    function purchaseAccess(bytes32 modelId, uint256 quantity) external payable;

    function updatePricing(
        bytes32 modelId,
        uint256 newBasePrice,
        uint256 newDiscountPrice,
        uint256 newMinimumBulkSize
    ) external;

    function updateCategory(bytes32 modelId, uint8 newCategory) external;

    function featureModel(bytes32 modelId) external payable;

    function unfeatureModel(bytes32 modelId) external;

    function addReview(bytes32 modelId, uint8 rating, string memory comment) external;

    function deactivateListing(bytes32 modelId) external;

    function activateListing(bytes32 modelId) external;

    // View functions
    function getListing(bytes32 modelId) external view returns (ModelListing memory);

    function getModelsByCategory(uint8 category) external view returns (bytes32[] memory);

    function getFeaturedModels() external view returns (bytes32[] memory);

    function getTopRatedModels(uint256 limit) external view returns (bytes32[] memory);

    function getModelsByOwner(address owner) external view returns (bytes32[] memory);

    function getPurchaseHistory(bytes32 modelId) external view returns (Purchase[] memory);

    function getModelReviews(bytes32 modelId) external view returns (Review[] memory);

    function getMarketplaceStats() external view returns (uint256, uint256, uint256);
}