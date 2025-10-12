// SPDX-License-Identifier: MIT

//lattice-v3/contracts/src/interfaces/IModelRegistry.sol
pragma solidity ^0.8.24;

interface IModelRegistry {
    struct ModelMetadata {
        string description;
        string[] inputShape;
        string[] outputShape;
        uint256 parameters;
        string license;
        string[] tags;
    }
    
    function registerModel(
        string memory name,
        string memory framework,
        string memory version,
        string memory ipfsCID,
        uint256 sizeBytes,
        uint256 inferencePrice,
        ModelMetadata memory metadata
    ) external payable returns (bytes32);
    
    function requestInference(
        bytes32 modelHash,
        bytes calldata inputData
    ) external payable returns (bytes memory);
    
    function hasPermission(bytes32 modelHash, address user) external view returns (bool);
    
    function getModel(bytes32 modelHash) external view returns (
        address owner,
        string memory name,
        string memory framework,
        string memory version,
        string memory ipfsCID,
        uint256 inferencePrice,
        uint256 totalInferences,
        bool isActive
    );
}