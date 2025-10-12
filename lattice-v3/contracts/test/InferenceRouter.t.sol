// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import {InferenceRouter} from "../src/InferenceRouter.sol";

contract InferenceRouterTest is Test {
    InferenceRouter router;

    function setUp() public {
        // Pass a dummy registry address; not used in these tests
        router = new InferenceRouter(address(0));
        // Lower the minimum stake for easier testing
        router.setMinProviderStake(1 ether);
    }

    function test_RegisterProvider_And_Request() public {
        address provider = address(0xA11CE);
        vm.deal(provider, 10 ether);

        // Register provider supporting a model
        bytes32[] memory models = new bytes32[](1);
        models[0] = keccak256("model-1");

        vm.prank(provider);
        router.registerProvider{value: 2 ether}("http://localhost:7000", 0.1 ether, models);

        address[] memory providers = router.getProviders(models[0]);
        assertEq(providers.length, 1);
        assertEq(providers[0], provider);

        // Enable caching
        router.setCaching(models[0], true);

        // First request: assigns provider and awaits completion
        address user = address(0xBEEF);
        vm.deal(user, 10 ether);
        vm.prank(user);
        uint256 reqId = router.requestInference{value: 1 ether}(models[0], hex"DEADBEEF", 1 ether);

        // Complete by provider
        vm.prank(provider);
        router.completeInference(reqId, hex"01");

        // Second request with same input: expect cache hit path to succeed
        vm.prank(user);
        uint256 cachedId = router.requestInference{value: 1 ether}(models[0], hex"DEADBEEF", 1 ether);
        ( , , InferenceRouter.RequestStatus status, bytes memory out, uint256 paid) = router.getRequest(cachedId);
        assertEq(uint(status), uint(InferenceRouter.RequestStatus.Completed));
        assertGt(out.length, 0);
        assertGt(paid, 0);
    }

    function test_Economics_FeesAndWithdrawal() public {
        // Two providers with different min prices
        address p1 = address(0xAAA1);
        address p2 = address(0xAAA2);
        vm.deal(p1, 5 ether);
        vm.deal(p2, 5 ether);

        bytes32 model = keccak256("model-eco");
        bytes32[] memory models = new bytes32[](1);
        models[0] = model;

        // Register providers
        vm.prank(p1);
        router.registerProvider{value: 2 ether}("http://p1", 0.2 ether, models);
        vm.prank(p2);
        router.registerProvider{value: 2 ether}("http://p2", 0.5 ether, models);

        // User requests at price ceiling 1 ether
        address user = address(0xC0FFEE);
        vm.deal(user, 10 ether);
        vm.prank(user);
        uint256 reqId = router.requestInference{value: 1 ether}(model, hex"BEEF", 1 ether);

        // Lower-price provider (p1) should be selected; completion should succeed from p1
        vm.prank(p1);
        router.completeInference(reqId, hex"01");

        // Provider earnings should reflect platform fee deduction
        // Default platform fee = 2.5% → provider gets 97.5% of minPrice
        uint256 expectedProvider = (0.2 ether * (10_000 - 250)) / 10_000;
        // Withdraw earnings
        uint256 balBefore = p1.balance;
        vm.prank(p1);
        router.withdrawEarnings();
        uint256 balAfter = p1.balance;
        assertEq(balAfter - balBefore, expectedProvider);
    }

    function test_Selection_Scoring_PicksLowerPrice() public {
        // Two providers, same stake/capacity but different min price
        address p1 = address(0xBB01);
        address p2 = address(0xBB02);
        vm.deal(p1, 3 ether);
        vm.deal(p2, 3 ether);

        bytes32 model = keccak256("model-route");
        bytes32[] memory models = new bytes32[](1);
        models[0] = model;

        // p1 cheaper than p2
        vm.prank(p1);
        router.registerProvider{value: 2 ether}("http://p1", 0.1 ether, models);
        vm.prank(p2);
        router.registerProvider{value: 2 ether}("http://p2", 0.9 ether, models);

        // User request
        address user = address(0xCAFE);
        vm.deal(user, 5 ether);
        vm.prank(user);
        uint256 reqId = router.requestInference{value: 1 ether}(model, hex"BEEF", 1 ether);

        // Completion from p1 (cheaper) should succeed; from p2 would revert
        vm.prank(p1);
        router.completeInference(reqId, hex"DEAD");
    }

    function test_RegisterProvider_RevertOnLowStake() public {
        bytes32[] memory models = new bytes32[](1);
        models[0] = keccak256("model-x");
        address provider = address(0xD00D);
        vm.deal(provider, 0.5 ether);
        vm.prank(provider);
        vm.expectRevert();
        router.registerProvider{value: 0.5 ether}("http://x", 0.1 ether, models);
    }

    function test_CancelRequest_OnlyRequester() public {
        // Register a provider
        address p = address(0xC001);
        vm.deal(p, 3 ether);
        bytes32[] memory models = new bytes32[](1);
        models[0] = keccak256("model-y");
        vm.prank(p);
        router.registerProvider{value: 2 ether}("http://p", 0.2 ether, models);

        // Create request - this will be immediately assigned to provider (status = Processing)
        address user = address(0xFEED);
        vm.deal(user, 2 ether);
        vm.prank(user);
        uint256 id = router.requestInference{value: 1 ether}(models[0], hex"AB", 1 ether);

        // Non-requester cannot cancel (should fail authorization first)
        vm.prank(p);
        vm.expectRevert(bytes("Not request owner"));
        router.cancelRequest(id);

        // Requester also cannot cancel because request is already Processing
        vm.prank(user);
        vm.expectRevert(bytes("Cannot cancel"));
        router.cancelRequest(id);
    }

    function test_Complete_RevertsIfNotAssignedProvider() public {
        address p1 = address(0x1111);
        address p2 = address(0x2222);
        vm.deal(p1, 3 ether);
        vm.deal(p2, 3 ether);
        bytes32[] memory models = new bytes32[](1);
        models[0] = keccak256("model-z");
        vm.prank(p1);
        router.registerProvider{value: 2 ether}("http://p1", 0.1 ether, models);
        vm.prank(p2);
        router.registerProvider{value: 2 ether}("http://p2", 0.2 ether, models);

        address user = address(0x3333);
        vm.deal(user, 3 ether);
        vm.prank(user);
        uint256 id = router.requestInference{value: 1 ether}(models[0], hex"AA", 1 ether);

        // Completion from wrong provider should revert
        vm.prank(p2);
        vm.expectRevert(bytes("Not assigned provider"));
        router.completeInference(id, hex"01");
    }

    function test_WithdrawStake_RequiresInactiveAndNoLoad() public {
        address p = address(0x4545);
        vm.deal(p, 3 ether);
        bytes32[] memory models = new bytes32[](1);
        models[0] = keccak256("model-w");
        vm.prank(p);
        router.registerProvider{value: 2 ether}("http://p", 0.1 ether, models);

        // Active -> cannot withdraw
        vm.prank(p);
        vm.expectRevert(bytes("Must deactivate first"));
        router.withdrawStake(1 ether);

        // Deactivate then withdraw
        vm.prank(p);
        router.updateProviderStatus(false);
        uint256 before = p.balance;
        vm.prank(p);
        router.withdrawStake(1 ether);
        assertEq(p.balance, before + 1 ether);
    }
}
