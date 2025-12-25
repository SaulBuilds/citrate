// SPDX-License-Identifier: MIT

// citrate-v3/contracts/test/ModelMarketplace.t.sol
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/ModelMarketplace.sol";
import "../src/ModelRegistry.sol";
import "../src/interfaces/IModelRegistry.sol";

contract ModelMarketplaceTest is Test {
    ModelMarketplace public marketplace;
    ModelRegistry public registry;

    address public owner = address(0x1001);
    address public treasury = address(0x1002);
    address public buyer = address(0x1003);
    address public reviewer = address(0x1004);

    bytes32 public testModelId;

    // Test model data
    string constant TEST_MODEL_NAME = "TestLLM";
    string constant TEST_FRAMEWORK = "CoreML";
    string constant TEST_VERSION = "1.0.0";
    string constant TEST_IPFS_CID = "QmTestCID123";
    uint256 constant TEST_SIZE = 1000000; // 1MB
    uint256 constant TEST_INFERENCE_PRICE = 0.01 ether;

    // Marketplace test data
    uint256 constant TEST_BASE_PRICE = 0.01 ether;
    uint256 constant TEST_DISCOUNT_PRICE = 0.008 ether;
    uint256 constant TEST_BULK_SIZE = 10;
    uint8 constant TEST_CATEGORY = 1; // Language Models
    string constant TEST_METADATA_URI = "ipfs://QmMarketplaceMetadata";

    function setUp() public {
        // Deploy contracts
        registry = new ModelRegistry();
        marketplace = new ModelMarketplace(address(registry), treasury);

        // Setup test accounts
        vm.deal(owner, 100 ether);
        vm.deal(buyer, 100 ether);
        vm.deal(reviewer, 100 ether);

        // Register a test model
        vm.startPrank(owner);

        IModelRegistry.ModelMetadata memory metadata = IModelRegistry.ModelMetadata({
            description: "Test language model",
            inputShape: new string[](1),
            outputShape: new string[](1),
            parameters: 7000000,
            license: "MIT",
            tags: new string[](2)
        });
        metadata.inputShape[0] = "512";
        metadata.outputShape[0] = "512";
        metadata.tags[0] = "language";
        metadata.tags[1] = "chat";

        testModelId = registry.registerModel{value: registry.REGISTRATION_FEE()}(
            TEST_MODEL_NAME,
            TEST_FRAMEWORK,
            TEST_VERSION,
            TEST_IPFS_CID,
            TEST_SIZE,
            TEST_INFERENCE_PRICE,
            metadata
        );

        vm.stopPrank();
    }

    function testListModel() public {
        vm.startPrank(owner);

        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);

        assertEq(listing.modelId, testModelId);
        assertEq(listing.owner, owner);
        assertEq(listing.basePrice, TEST_BASE_PRICE);
        assertEq(listing.discountPrice, TEST_DISCOUNT_PRICE);
        assertEq(listing.minimumBulkSize, TEST_BULK_SIZE);
        assertEq(listing.category, TEST_CATEGORY);
        assertEq(listing.metadataURI, TEST_METADATA_URI);
        assertTrue(listing.active);
        assertFalse(listing.featured);

        vm.stopPrank();
    }

    function testListModelFailsForNonOwner() public {
        vm.startPrank(buyer);

        vm.expectRevert("Not model owner");
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        vm.stopPrank();
    }

    function testListModelFailsInvalidPrice() public {
        vm.startPrank(owner);

        vm.expectRevert("Invalid price");
        marketplace.listModel(
            testModelId,
            0.0001 ether, // Below minimum
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        vm.expectRevert("Invalid price");
        marketplace.listModel(
            testModelId,
            2000 ether, // Above maximum
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        vm.stopPrank();
    }

    function testPurchaseAccess() public {
        // First list the model
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        uint256 quantity = 5;
        uint256 totalPrice = TEST_BASE_PRICE * quantity;
        uint256 expectedFee = (totalPrice * marketplace.MARKETPLACE_FEE_BASIS_POINTS()) / 10000;
        uint256 expectedSellerAmount = totalPrice - expectedFee;

        uint256 ownerBalanceBefore = owner.balance;
        uint256 treasuryBalanceBefore = treasury.balance;

        vm.startPrank(buyer);
        marketplace.purchaseAccess{value: totalPrice}(testModelId, quantity);
        vm.stopPrank();

        // Check balances
        assertEq(owner.balance, ownerBalanceBefore + expectedSellerAmount);
        assertEq(treasury.balance, treasuryBalanceBefore + expectedFee);

        // Check listing stats
        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);
        assertEq(listing.totalSales, quantity);
        assertEq(listing.totalRevenue, totalPrice);

        // Check purchase tracking
        assertEq(marketplace.userPurchases(buyer, testModelId), quantity);
    }

    function testPurchaseAccessBulkDiscount() public {
        // First list the model
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        uint256 quantity = 15; // Above bulk size
        uint256 totalPrice = TEST_DISCOUNT_PRICE * quantity; // Should use discount price

        vm.startPrank(buyer);
        marketplace.purchaseAccess{value: totalPrice}(testModelId, quantity);
        vm.stopPrank();

        // Check purchase history
        ModelMarketplace.Purchase[] memory purchases = marketplace.getPurchaseHistory(testModelId);
        assertEq(purchases.length, 1);
        assertEq(purchases[0].price, TEST_DISCOUNT_PRICE);
        assertTrue(purchases[0].bulkDiscount);
    }

    function testPurchaseAccessFailsInsufficientPayment() public {
        // First list the model
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        uint256 quantity = 5;
        uint256 insufficientPayment = (TEST_BASE_PRICE * quantity) - 1 wei;

        vm.startPrank(buyer);
        vm.expectRevert("Insufficient payment");
        marketplace.purchaseAccess{value: insufficientPayment}(testModelId, quantity);
        vm.stopPrank();
    }

    function testUpdatePricing() public {
        // First list the model
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        uint256 newBasePrice = 0.02 ether;
        uint256 newDiscountPrice = 0.015 ether;
        uint256 newBulkSize = 5;

        marketplace.updatePricing(testModelId, newBasePrice, newDiscountPrice, newBulkSize);

        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);
        assertEq(listing.basePrice, newBasePrice);
        assertEq(listing.discountPrice, newDiscountPrice);
        assertEq(listing.minimumBulkSize, newBulkSize);

        vm.stopPrank();
    }

    function testFeatureModel() public {
        // First list the model
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        // Owner can pay to feature
        marketplace.featureModel{value: marketplace.FEATURED_FEE()}(testModelId);

        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);
        assertTrue(listing.featured);

        bytes32[] memory featuredModels = marketplace.getFeaturedModels();
        assertEq(featuredModels.length, 1);
        assertEq(featuredModels[0], testModelId);

        vm.stopPrank();
    }

    function testAddReview() public {
        // First list and purchase
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        vm.startPrank(buyer);
        marketplace.purchaseAccess{value: TEST_BASE_PRICE * 5}(testModelId, 5);

        // Add a review
        uint8 rating = 4;
        string memory comment = "Great model!";
        marketplace.addReview(testModelId, rating, comment);
        vm.stopPrank();

        // Check review was added
        ModelMarketplace.Review[] memory reviews = marketplace.getModelReviews(testModelId);
        assertEq(reviews.length, 1);
        assertEq(reviews[0].reviewer, buyer);
        assertEq(reviews[0].rating, rating);
        assertEq(reviews[0].comment, comment);
        assertTrue(reviews[0].verified); // Should be verified since buyer purchased

        // Check rating stats
        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);
        assertEq(listing.reviewCount, 1);
        assertEq(listing.totalRating, rating);
        assertEq(listing.averageRating, uint256(rating) * 100); // Stored as percentage
    }

    function testAddReviewUnverified() public {
        // List model but don't purchase
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        // Add review without purchasing
        vm.startPrank(reviewer);
        marketplace.addReview(testModelId, 3, "Haven't used it but looks good");
        vm.stopPrank();

        ModelMarketplace.Review[] memory reviews = marketplace.getModelReviews(testModelId);
        assertEq(reviews.length, 1);
        assertFalse(reviews[0].verified); // Should not be verified
    }

    function testAddReviewFailsInvalidRating() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        vm.startPrank(buyer);
        vm.expectRevert("Rating must be between 1 and 5");
        marketplace.addReview(testModelId, 0, "Invalid rating");

        vm.expectRevert("Rating must be between 1 and 5");
        marketplace.addReview(testModelId, 6, "Invalid rating");
        vm.stopPrank();
    }

    function testAddReviewFailsDuplicate() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        vm.startPrank(buyer);
        marketplace.addReview(testModelId, 5, "First review");

        vm.expectRevert("Already reviewed this model");
        marketplace.addReview(testModelId, 4, "Second review");
        vm.stopPrank();
    }

    function testGetModelsByCategory() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        bytes32[] memory categoryModels = marketplace.getModelsByCategory(TEST_CATEGORY);
        assertEq(categoryModels.length, 1);
        assertEq(categoryModels[0], testModelId);
    }

    function testUpdateCategory() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        uint8 newCategory = 2;
        marketplace.updateCategory(testModelId, newCategory);

        // Check old category is empty
        bytes32[] memory oldCategoryModels = marketplace.getModelsByCategory(TEST_CATEGORY);
        assertEq(oldCategoryModels.length, 0);

        // Check new category has the model
        bytes32[] memory newCategoryModels = marketplace.getModelsByCategory(newCategory);
        assertEq(newCategoryModels.length, 1);
        assertEq(newCategoryModels[0], testModelId);

        vm.stopPrank();
    }

    function testDeactivateAndActivateListing() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );

        // Deactivate
        marketplace.deactivateListing(testModelId);
        ModelMarketplace.ModelListing memory listing = marketplace.getListing(testModelId);
        assertFalse(listing.active);

        // Activate
        marketplace.activateListing(testModelId);
        listing = marketplace.getListing(testModelId);
        assertTrue(listing.active);

        vm.stopPrank();
    }

    function testGetMarketplaceStats() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        vm.startPrank(buyer);
        uint256 quantity = 5;
        uint256 totalPrice = TEST_BASE_PRICE * quantity;
        marketplace.purchaseAccess{value: totalPrice}(testModelId, quantity);
        vm.stopPrank();

        (uint256 totalListings, uint256 totalSales, uint256 totalVolume) = marketplace.getMarketplaceStats();

        assertEq(totalListings, 1);
        assertEq(totalSales, quantity);
        assertEq(totalVolume, totalPrice);
    }

    function testExcessPaymentRefund() public {
        vm.startPrank(owner);
        marketplace.listModel(
            testModelId,
            TEST_BASE_PRICE,
            TEST_DISCOUNT_PRICE,
            TEST_BULK_SIZE,
            TEST_CATEGORY,
            TEST_METADATA_URI
        );
        vm.stopPrank();

        uint256 quantity = 5;
        uint256 exactPrice = TEST_BASE_PRICE * quantity;
        uint256 excessPayment = exactPrice + 1 ether;

        uint256 buyerBalanceBefore = buyer.balance;

        vm.startPrank(buyer);
        marketplace.purchaseAccess{value: excessPayment}(testModelId, quantity);
        vm.stopPrank();

        // Buyer should receive refund for excess payment
        uint256 expectedBalance = buyerBalanceBefore - exactPrice;
        assertEq(buyer.balance, expectedBalance);
    }
}