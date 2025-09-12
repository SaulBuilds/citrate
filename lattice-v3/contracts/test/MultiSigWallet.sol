// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MultiSigWallet
 * @dev Simple multi-signature wallet for testing complex transactions
 */
contract MultiSigWallet {
    struct Transaction {
        address to;
        uint256 value;
        bytes data;
        bool executed;
        uint256 confirmations;
    }
    
    address[] public owners;
    mapping(address => bool) public isOwner;
    uint256 public requiredConfirmations;
    
    Transaction[] public transactions;
    mapping(uint256 => mapping(address => bool)) public confirmations;
    
    event TransactionSubmitted(uint256 indexed txId, address indexed submitter);
    event TransactionConfirmed(uint256 indexed txId, address indexed confirmer);
    event TransactionExecuted(uint256 indexed txId, address indexed executor);
    event TransactionRevoked(uint256 indexed txId, address indexed revoker);
    event Deposit(address indexed sender, uint256 value);
    
    modifier onlyOwner() {
        require(isOwner[msg.sender], "Not an owner");
        _;
    }
    
    modifier txExists(uint256 _txId) {
        require(_txId < transactions.length, "Transaction does not exist");
        _;
    }
    
    modifier notExecuted(uint256 _txId) {
        require(!transactions[_txId].executed, "Transaction already executed");
        _;
    }
    
    constructor(address[] memory _owners, uint256 _requiredConfirmations) {
        require(_owners.length > 0, "Owners required");
        require(_requiredConfirmations > 0 && _requiredConfirmations <= _owners.length,
                "Invalid required confirmations");
        
        for (uint256 i = 0; i < _owners.length; i++) {
            address owner = _owners[i];
            require(owner != address(0), "Invalid owner");
            require(!isOwner[owner], "Duplicate owner");
            
            isOwner[owner] = true;
            owners.push(owner);
        }
        
        requiredConfirmations = _requiredConfirmations;
    }
    
    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }
    
    function submitTransaction(address _to, uint256 _value, bytes memory _data)
        public
        onlyOwner
        returns (uint256)
    {
        uint256 txId = transactions.length;
        
        transactions.push(Transaction({
            to: _to,
            value: _value,
            data: _data,
            executed: false,
            confirmations: 0
        }));
        
        emit TransactionSubmitted(txId, msg.sender);
        confirmTransaction(txId);
        
        return txId;
    }
    
    function confirmTransaction(uint256 _txId)
        public
        onlyOwner
        txExists(_txId)
        notExecuted(_txId)
    {
        require(!confirmations[_txId][msg.sender], "Already confirmed");
        
        confirmations[_txId][msg.sender] = true;
        transactions[_txId].confirmations++;
        
        emit TransactionConfirmed(_txId, msg.sender);
        
        if (transactions[_txId].confirmations >= requiredConfirmations) {
            executeTransaction(_txId);
        }
    }
    
    function executeTransaction(uint256 _txId)
        public
        onlyOwner
        txExists(_txId)
        notExecuted(_txId)
    {
        Transaction storage transaction = transactions[_txId];
        
        require(transaction.confirmations >= requiredConfirmations,
                "Not enough confirmations");
        
        transaction.executed = true;
        
        (bool success, ) = transaction.to.call{value: transaction.value}(transaction.data);
        require(success, "Transaction execution failed");
        
        emit TransactionExecuted(_txId, msg.sender);
    }
    
    function revokeConfirmation(uint256 _txId)
        public
        onlyOwner
        txExists(_txId)
        notExecuted(_txId)
    {
        require(confirmations[_txId][msg.sender], "Not confirmed");
        
        confirmations[_txId][msg.sender] = false;
        transactions[_txId].confirmations--;
        
        emit TransactionRevoked(_txId, msg.sender);
    }
    
    function getTransactionCount() public view returns (uint256) {
        return transactions.length;
    }
    
    function getOwners() public view returns (address[] memory) {
        return owners;
    }
}