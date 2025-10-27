// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/ModelRegistry.sol";
import "../src/InferenceRouter.sol";
import "../src/LoRAFactory.sol";

contract DeployScript is Script {
    function run() external {
        // Get private key from environment variable
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy ModelRegistry
        ModelRegistry modelRegistry = new ModelRegistry();
        console.log("ModelRegistry deployed at:", address(modelRegistry));
        
        // Deploy InferenceRouter with ModelRegistry address
        InferenceRouter inferenceRouter = new InferenceRouter(address(modelRegistry));
        console.log("InferenceRouter deployed at:", address(inferenceRouter));
        
        // Deploy LoRAFactory with ModelRegistry address
        LoRAFactory loraFactory = new LoRAFactory(address(modelRegistry));
        console.log("LoRAFactory deployed at:", address(loraFactory));
        
        vm.stopBroadcast();
        
        // Log deployment addresses for reference
        console.log("\n=== Deployment Summary ===");
        console.log("ModelRegistry:", address(modelRegistry));
        console.log("InferenceRouter:", address(inferenceRouter));
        console.log("LoRAFactory:", address(loraFactory));
    }
}