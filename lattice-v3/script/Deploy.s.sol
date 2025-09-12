// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../contracts/test/SimpleStorage.sol";
import "../contracts/test/Token.sol";
import "../contracts/test/MultiSigWallet.sol";

contract DeployScript is Script {
    function run() external {
        // Use a test private key (DO NOT USE IN PRODUCTION)
        uint256 deployerPrivateKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy SimpleStorage
        SimpleStorage storage_ = new SimpleStorage(42);
        console.log("SimpleStorage deployed at:", address(storage_));
        
        // Deploy TestToken with 1 million tokens
        TestToken token = new TestToken(1000000 * 10**18);
        console.log("TestToken deployed at:", address(token));
        
        // Deploy MultiSigWallet with single owner for testing
        address[] memory owners = new address[](1);
        owners[0] = vm.addr(deployerPrivateKey);
        MultiSigWallet wallet = new MultiSigWallet(owners, 1);
        console.log("MultiSigWallet deployed at:", address(wallet));
        
        vm.stopBroadcast();
    }
}
