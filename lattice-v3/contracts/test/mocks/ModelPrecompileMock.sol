// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract ModelPrecompileMock {
    bytes32 public lastModelHash;
    string public lastIpfsCID;
    bytes public lastInput;

    event Registered(bytes32 indexed modelHash, string ipfsCID);
    event Inference(bytes32 indexed modelHash, bytes input);

    function registerModel(bytes32 modelHash, string memory ipfsCID) external {
        lastModelHash = modelHash;
        lastIpfsCID = ipfsCID;
        emit Registered(modelHash, ipfsCID);
    }

    function executeInference(bytes32 modelHash, bytes calldata input) external returns (bytes memory) {
        lastModelHash = modelHash;
        lastInput = input;
        emit Inference(modelHash, input);
        // Return a simple echo-like payload to assert on
        return bytes.concat(bytes("out:"), input);
    }
}

