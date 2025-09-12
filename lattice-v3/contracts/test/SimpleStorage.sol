// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title SimpleStorage
 * @dev Simple storage contract for testing basic functionality
 */
contract SimpleStorage {
    uint256 public storedValue;
    address public lastSender;
    
    event ValueChanged(uint256 oldValue, uint256 newValue, address indexed changer);
    event MessageLogged(string message, address indexed sender);
    
    constructor(uint256 _initialValue) {
        storedValue = _initialValue;
        lastSender = msg.sender;
    }
    
    function setValue(uint256 _newValue) public {
        uint256 oldValue = storedValue;
        storedValue = _newValue;
        lastSender = msg.sender;
        emit ValueChanged(oldValue, _newValue, msg.sender);
    }
    
    function getValue() public view returns (uint256) {
        return storedValue;
    }
    
    function increment() public {
        storedValue++;
        lastSender = msg.sender;
        emit ValueChanged(storedValue - 1, storedValue, msg.sender);
    }
    
    function logMessage(string memory _message) public {
        emit MessageLogged(_message, msg.sender);
    }
}