// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {ModelRegistry} from "../src/ModelRegistry.sol";
import {IModelRegistry} from "../src/interfaces/IModelRegistry.sol";
import {ModelPrecompileMock} from "./mocks/ModelPrecompileMock.sol";

contract ModelRegistryTest is Test {
    ModelRegistry internal registry;
    address internal owner = address(0xAA11);
    address internal user = address(0xBB22);

    // Citrate precompile address used by ModelRegistry
    address constant MODEL_PRECOMPILE = 0x0000000000000000000000000000000000001000;

    function setUp() public {
        registry = new ModelRegistry();

        // Deploy mock precompile and etch its runtime code at the precompile address
        ModelPrecompileMock mock = new ModelPrecompileMock();
        bytes memory code = address(mock).code;
        vm.etch(MODEL_PRECOMPILE, code);

        // Fund default actors
        vm.deal(owner, 100 ether);
        vm.deal(user, 100 ether);
    }

    function _meta() internal pure returns (IModelRegistry.ModelMetadata memory m) {
        string[] memory inShape = new string[](1);
        inShape[0] = "1x3x224x224";
        string[] memory outShape = new string[](1);
        outShape[0] = "1x1000";
        string[] memory tags = new string[](2);
        tags[0] = "vision";
        tags[1] = "imagenet";
        m = IModelRegistry.ModelMetadata({
            description: "ResNet50",
            inputShape: inShape,
            outputShape: outShape,
            parameters: 25557032,
            license: "Apache-2.0",
            tags: tags
        });
    }

    function test_registerModel_success() public {
        vm.prank(owner);
        bytes32 hash = registry.registerModel{value: 0.1 ether}(
            "ResNet",
            "PyTorch",
            "1.0.0",
            "bafyCID",
            10_000_000,
            0.01 ether,
            _meta()
        );

        (address o, string memory name,,, string memory cid, uint256 price,, bool isActive) = registry.getModel(hash);
        assertEq(o, owner);
        assertEq(name, "ResNet");
        assertEq(cid, "bafyCID");
        assertEq(price, 0.01 ether);
        assertTrue(isActive);

        bytes32[] memory mine = registry.getModelsByOwner(owner);
        assertEq(mine.length, 1);
        assertEq(mine[0], hash);
    }

    function test_registerModel_revertsOnMissingFee() public {
        vm.prank(owner);
        vm.expectRevert(bytes("Insufficient registration fee"));
        registry.registerModel(
            "X","Fw","1","cid",123, 1 ether,_meta()
        );
    }

    function test_registerModel_revertsOnEmptyName() public {
        vm.prank(owner);
        vm.expectRevert(bytes("Name required"));
        registry.registerModel{value:0.1 ether}("","Fw","1","cid",123, 0,_meta());
    }

    function test_registerModel_revertsOnEmptyIpfs() public {
        vm.prank(owner);
        vm.expectRevert(bytes("IPFS CID required"));
        registry.registerModel{value:0.1 ether}("N","Fw","1","",123, 0,_meta());
    }

    function test_activate_deactivate_and_permission_checks() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 0.2 ether,_meta());

        // Deactivate by owner blocks inference
        vm.prank(owner);
        registry.deactivateModel(h);
        vm.prank(user);
        vm.expectRevert(bytes("Model not active"));
        registry.requestInference{value:0.2 ether}(h, hex"00");

        // Reactivate by operator (owner has DEFAULT_ADMIN_ROLE and OPERATOR_ROLE)
        vm.prank(owner);
        registry.activateModel(h);

        // Without permission (paid model) request should revert
        vm.prank(user);
        vm.expectRevert(bytes("No permission"));
        registry.requestInference{value:0.2 ether}(h, hex"01");

        // Grant permission then inference should succeed
        vm.prank(owner);
        registry.grantPermission(h, user);
        vm.prank(user);
        bytes memory out = registry.requestInference{value:0.2 ether}(h, hex"01");
        assertGt(out.length, 0);
    }

    function test_requestInference_revertsOnInsufficientPayment() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 1 ether,_meta());
        vm.prank(owner);
        registry.grantPermission(h, user);
        vm.prank(user);
        vm.expectRevert(bytes("Insufficient payment"));
        registry.requestInference{value:0.5 ether}(h, hex"00");
    }

    function test_updateModel_onlyOwner_and_active() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 0.1 ether,_meta());

        vm.prank(user);
        vm.expectRevert(bytes("Not model owner"));
        registry.updateModel(h, "2", "cidB");

        vm.prank(owner);
        registry.updateModel(h, "2", "cidB");
        (, , , string memory v, string memory cid, , , ) = registry.getModel(h);
        assertEq(v, "2");
        assertEq(cid, "cidB");
    }

    function test_setInferencePrice_onlyOwner() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 1 ether,_meta());
        vm.prank(user);
        vm.expectRevert(bytes("Not model owner"));
        registry.setInferencePrice(h, 2 ether);

        vm.prank(owner);
        registry.setInferencePrice(h, 2 ether);
        (,,,,,, uint256 totalInf, bool active) = registry.getModel(h);
        assertEq(totalInf, 0);
        assertTrue(active);
        // spot-check via inference price read path
        (, , , , , uint256 price, , ) = registry.getModel(h);
        assertEq(price, 2 ether);
    }

    function test_permissions_grant_revoke() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 0.5 ether,_meta());
        assertFalse(registry.hasPermission(h, user));

        vm.prank(owner);
        registry.grantPermission(h, user);
        assertTrue(registry.hasPermission(h, user));

        vm.prank(owner);
        registry.revokePermission(h, user);
        assertFalse(registry.hasPermission(h, user));
    }

    function test_requestInference_enforcesPaymentAndPermissions() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 1 ether,_meta());

        // Without permission should fail (price > 0)
        vm.prank(user);
        vm.expectRevert(bytes("No permission"));
        registry.requestInference{value:1 ether}(h, hex"1122");

        // Grant permission, but insufficient payment
        vm.prank(owner);
        registry.grantPermission(h, user);
        vm.prank(user);
        vm.expectRevert(bytes("Insufficient payment"));
        registry.requestInference{value:0.5 ether}(h, hex"1122");

        // Pay exact price, expect owner to receive funds and stats to increment
        uint256 balBefore = owner.balance;
        vm.prank(user);
        bytes memory out = registry.requestInference{value:1 ether}(h, hex"AABB");
        assertGt(owner.balance, balBefore);
        // Output is prefixed by mock and ABI-encoded
        assertEq(out, abi.encode(bytes.concat(bytes("out:"), hex"AABB")));
        assertEq(registry.getModelRevenue(h), 1 ether);
    }

    function test_requestInference_freeModel_noPermissionNeeded() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("Free","Fw","1","cidA",123, 0,_meta());
        vm.prank(user);
        bytes memory out = registry.requestInference{value:0}(h, hex"00");
        assertEq(out, abi.encode(bytes("out:\x00")));
    }

    function test_activate_deactivate_authorization() public {
        vm.prank(owner);
        bytes32 h = registry.registerModel{value:0.1 ether}("N","Fw","1","cidA",123, 0,_meta());

        // non-owner & non-operator should fail
        vm.prank(user);
        vm.expectRevert(bytes("Not authorized"));
        registry.deactivateModel(h);

        // owner can deactivate / activate
        vm.prank(owner);
        registry.deactivateModel(h);
        (, , , , , , , bool active0) = registry.getModel(h);
        assertFalse(active0);

        vm.prank(owner);
        registry.activateModel(h);
        (, , , , , , , bool active1) = registry.getModel(h);
        assertTrue(active1);
    }

    function testFuzz_registerAndGet(address who, uint256 size, uint256 price) public {
        vm.assume(who != address(0));
        vm.deal(who, 10 ether);
        vm.prank(who);
        bytes32 h = registry.registerModel{value:0.1 ether}(
            "N","F","1","cid", size % 1e12, price % 10 ether, _meta()
        );
        (address o,,,,,, , bool active) = registry.getModel(h);
        assertEq(o, who);
        assertTrue(active);
        bytes32[] memory ow = registry.getModelsByOwner(who);
        assertEq(ow.length, 1);
        assertEq(ow[0], h);
    }

    // Pen-test: reentrancy blocked via nonReentrant; owner receive() cannot re-enter on paid model
    function testPen_ReentrancyBlockedOnPaidModel() public {
        // Deploy a reentrant owner to receive payment callback
        ReentrantOwner ro = new ReentrantOwner(registry);
        vm.deal(address(ro), 3 ether); // Need extra for reentrancy attempt

        // Reentrant owner registers a paid model (payment will trigger receive())
        vm.prank(address(ro));
        bytes32 h = registry.registerModel{value:0.1 ether}("R","F","1","cid",1, 1 ether,_meta());
        ro.set(h);

        // Grant permission to a user
        vm.prank(address(ro));
        registry.grantPermission(h, user);

        // User requests inference with exact payment; expect revert due to payment failure (caused by reentrancy)
        vm.prank(user);
        vm.expectRevert(bytes("Payment failed"));
        registry.requestInference{value:1 ether}(h, hex"01");
    }
}

contract ReentrantOwner {
    ModelRegistry private reg;
    bytes32 public lastModelHash;
    constructor(ModelRegistry _reg) { reg = _reg; }
    receive() external payable {
        // Re-enter with sufficient payment to trigger reentrancy guard
        if (lastModelHash != bytes32(0) && address(this).balance >= 1 ether) {
            bytes32 modelToCall = lastModelHash;
            lastModelHash = bytes32(0); // Prevent infinite recursion
            reg.requestInference{value:1 ether}(modelToCall, hex"02");
        }
    }
    function set(bytes32 h) external { lastModelHash = h; }
}
