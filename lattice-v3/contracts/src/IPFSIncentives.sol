// SPDX-License-Identifier: MIT

// lattice-v3/contracts/src/IPFSIncentives.sol
pragma solidity ^0.8.24;

import "./lib/AccessControl.sol";
import "./lib/ReentrancyGuard.sol";

/**
 * @title IPFSIncentives
 * @notice Tracks storage providers pinning model artifacts and rewards them in native LATT.
 * @dev Rewards are denominated in wei. Contract must be pre-funded by an admin to pay out claims.
 */
contract IPFSIncentives is AccessControl, ReentrancyGuard {
    bytes32 public constant REPORTER_ROLE = keccak256("REPORTER_ROLE");
    uint256 public constant BYTES_PER_GB = 1_000_000_000;

    /// Reward paid per gigabyte pinned (before multipliers), defaults to 1 LATT.
    uint256 public baseRewardPerGb = 1 ether;
    uint256 public totalPendingRewards;

    enum ModelType {
        LANGUAGE,
        VISION,
        AUDIO,
        MULTIMODAL,
        CUSTOM
    }

    struct PinnerInfo {
        uint256 totalPinned;
        uint256 rewardsEarned;
        uint64 reports;
        bool exists;
    }

    struct ModelPinStats {
        uint256 totalPinned;
        uint256 totalRewards;
        uint8 modelType;
        bool initialised;
    }

    // Provider totals across all models
    mapping(address => uint256) public pinnedStorage;

    // Per-CID statistics
    mapping(string => ModelPinStats) public modelStats;
    mapping(string => address[]) private modelPinnerList;
    mapping(string => mapping(address => PinnerInfo)) public pinnerStats;

    // Pending rewards per provider
    mapping(address => uint256) public pendingRewards;

    event PinReported(string indexed cid, address indexed reporter, uint256 size, uint256 reward);
    event RewardClaimed(address indexed reporter, uint256 amount);
    event BaseRewardUpdated(uint256 newRate);
    event RewardsDeposited(address indexed funder, uint256 amount);

    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(REPORTER_ROLE, msg.sender);
    }

    /**
     * @notice Report that the caller is pinning the specified CID.
     * @param cid IPFS content identifier.
     * @param sizePinned Size of the data pinned in bytes.
     * @param modelType Model classification used to calculate multipliers.
     */
    function reportPinning(
        string calldata cid,
        uint256 sizePinned,
        ModelType modelType
    ) external onlyRole(REPORTER_ROLE) {
        require(sizePinned > 0, "Size required");

        ModelPinStats storage stats = modelStats[cid];
        if (!stats.initialised) {
            stats.initialised = true;
            stats.modelType = uint8(modelType);
        }

        uint256 reward = calculateReward(sizePinned, modelType);
        require(address(this).balance >= totalPendingRewards + reward, "Insufficient funding");

        stats.totalPinned += sizePinned;
        stats.totalRewards += reward;

        PinnerInfo storage info = pinnerStats[cid][msg.sender];
        if (!info.exists) {
            info.exists = true;
            modelPinnerList[cid].push(msg.sender);
        }

        info.totalPinned += sizePinned;
        info.rewardsEarned += reward;
        info.reports += 1;

        pinnedStorage[msg.sender] += sizePinned;
        pendingRewards[msg.sender] += reward;
        totalPendingRewards += reward;

        emit PinReported(cid, msg.sender, sizePinned, reward);
    }

    /**
     * @notice Claim previously accrued rewards.
     */
    function claimRewards() external nonReentrant {
        uint256 amount = pendingRewards[msg.sender];
        require(amount > 0, "No rewards");
        pendingRewards[msg.sender] = 0;
        totalPendingRewards -= amount;

        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");

        emit RewardClaimed(msg.sender, amount);
    }

    /**
     * @notice Update the base reward paid per gigabyte.
     * @param newRate Reward in wei per gigabyte before multipliers.
     */
    function updateBaseReward(uint256 newRate) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newRate > 0, "Invalid rate");
        baseRewardPerGb = newRate;
        emit BaseRewardUpdated(newRate);
    }

    /**
     * @notice Deposit additional funds so rewards can be paid out.
     */
    function depositRewards() external payable onlyRole(DEFAULT_ADMIN_ROLE) {
        require(msg.value > 0, "Amount required");
        emit RewardsDeposited(msg.sender, msg.value);
    }

    /**
     * @notice View helper returning the active pinners for a CID.
     */
    function getModelPinners(string calldata cid) external view returns (address[] memory) {
        return modelPinnerList[cid];
    }

    /**
     * @notice Calculate the reward that would be issued for `sizePinned` bytes.
     */
    function calculateReward(uint256 sizePinned, ModelType modelType) public view returns (uint256) {
        uint256 sizeGb = (sizePinned + BYTES_PER_GB - 1) / BYTES_PER_GB;
        if (sizeGb == 0) {
            sizeGb = 1;
        }
        uint256 multiplier = _rewardMultiplier(modelType);
        return sizeGb * multiplier * baseRewardPerGb;
    }

    function _rewardMultiplier(ModelType modelType) internal pure returns (uint256) {
        if (modelType == ModelType.LANGUAGE) {
            return 2;
        } else if (modelType == ModelType.VISION) {
            return 3;
        } else if (modelType == ModelType.AUDIO) {
            return 2;
        } else if (modelType == ModelType.MULTIMODAL) {
            return 4;
        }
        return 1;
    }

    receive() external payable {
        emit RewardsDeposited(msg.sender, msg.value);
    }
}
