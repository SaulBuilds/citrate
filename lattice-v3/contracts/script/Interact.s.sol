// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/ModelRegistry.sol";
import "../src/InferenceRouter.sol";
import "../src/LoRAFactory.sol";
import "../src/interfaces/IModelRegistry.sol";

contract InteractScript is Script {
    function run() external {
        // Get deployed contract addresses from environment
        address modelRegistryAddr = vm.envAddress("MODEL_REGISTRY");
        address inferenceRouterAddr = vm.envAddress("INFERENCE_ROUTER");
        address loraFactoryAddr = vm.envAddress("LORA_FACTORY");
        
        // Get private key
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Get contract instances
        ModelRegistry modelRegistry = ModelRegistry(modelRegistryAddr);
        InferenceRouter inferenceRouter = InferenceRouter(inferenceRouterAddr);
        LoRAFactory loraFactory = LoRAFactory(loraFactoryAddr);
        
        // Register a test model
        console.log("Registering test model...");
        
        IModelRegistry.ModelMetadata memory metadata = IModelRegistry.ModelMetadata({
            description: "GPT-2 Small Model for testing",
            inputShape: new string[](2),
            outputShape: new string[](2),
            parameters: 124000000, // 124M parameters
            license: "MIT",
            tags: new string[](3)
        });
        
        metadata.inputShape[0] = "batch_size";
        metadata.inputShape[1] = "sequence_length";
        metadata.outputShape[0] = "batch_size";
        metadata.outputShape[1] = "vocab_size";
        metadata.tags[0] = "language-model";
        metadata.tags[1] = "gpt2";
        metadata.tags[2] = "test";
        
        bytes32 modelHash = modelRegistry.registerModel{value: 0.1 ether}(
            "GPT-2-Small-Test",
            "PyTorch",
            "1.0.0",
            "QmTestModelCID123456789", // Mock IPFS CID
            536870912, // 512 MB
            0.001 ether, // 0.001 LATT per inference
            metadata
        );
        
        console.log("Model registered with hash:", vm.toString(modelHash));
        
        // Register as compute provider
        console.log("\nRegistering as compute provider...");
        
        bytes32[] memory supportedModels = new bytes32[](1);
        supportedModels[0] = modelHash;
        
        inferenceRouter.registerProvider{value: 100 ether}(
            "http://localhost:8080/inference",
            0.0005 ether, // Min price
            supportedModels
        );
        
        console.log("Registered as compute provider");
        
        // Create a LoRA adapter
        console.log("\nCreating LoRA adapter...");
        
        LoRAFactory.TrainingConfig memory trainingConfig = LoRAFactory.TrainingConfig({
            epochs: 10,
            batchSize: 32,
            learningRate: 1e15, // 0.001 in fixed point
            datasetCID: "QmTestDatasetCID123",
            datasetSize: 10000,
            validationSplit: 2000 // 20%
        });
        
        bytes32 loraHash = loraFactory.createLoRA{value: 0.1 ether}(
            modelHash,
            "Fine-tuned for Code",
            "LoRA adapter fine-tuned on code dataset",
            4, // rank
            16, // alpha
            100, // 1% dropout
            trainingConfig
        );
        
        console.log("LoRA created with hash:", vm.toString(loraHash));
        
        // Request inference
        console.log("\nRequesting inference...");
        
        bytes memory inputData = abi.encode("Test prompt for inference");
        
        uint256 requestId = inferenceRouter.requestInference{value: 0.002 ether}(
            modelHash,
            inputData,
            0.002 ether
        );
        
        console.log("Inference requested with ID:", requestId);
        
        vm.stopBroadcast();
        
        console.log("\n=== Interaction Complete ===");
    }
}