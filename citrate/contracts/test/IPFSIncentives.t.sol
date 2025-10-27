// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {IPFSIncentives} from "../src/IPFSIncentives.sol";

contract IPFSIncentivesTest is Test {
    IPFSIncentives internal incentives;
    address internal reporter = address(0xBEEF);

    function setUp() public {
        incentives = new IPFSIncentives();
        vm.deal(address(this), 20 ether);

        // Admin grants reporter role and funds the contract.
        incentives.grantRole(incentives.REPORTER_ROLE(), reporter);
        incentives.depositRewards{value: 10 ether}(); // Increased funding to cover rewards
    }

    function test_reportPinningAccruesRewards() public {
        vm.prank(reporter);
        incentives.reportPinning(
            "bafyModelCID",
            1_073_741_824, // 1GB
            IPFSIncentives.ModelType.VISION
        );

        uint256 pending = incentives.pendingRewards(reporter);
        assertGt(pending, 0);

        address[] memory pinners = incentives.getModelPinners("bafyModelCID");
        assertEq(pinners.length, 1);
        assertEq(pinners[0], reporter);

        uint256 beforeBalance = reporter.balance;
        vm.startPrank(reporter);
        incentives.claimRewards();
        vm.stopPrank();

        assertGt(reporter.balance, beforeBalance);
    }

    function test_updateBaseRewardAffectsCalculation() public {
        uint256 defaultReward = incentives.calculateReward(
            1_073_741_824,
            IPFSIncentives.ModelType.LANGUAGE
        );
        incentives.updateBaseReward(2 ether);
        uint256 updatedReward = incentives.calculateReward(
            1_073_741_824,
            IPFSIncentives.ModelType.LANGUAGE
        );

        assertGt(updatedReward, defaultReward);
    }
}
