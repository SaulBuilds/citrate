// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TestToken
 * @dev Simple ERC20-like token for testing transfers and balances
 */
contract TestToken {
    mapping(address => uint256) public balances;
    mapping(address => mapping(address => uint256)) public allowances;
    
    string public name = "Test Token";
    string public symbol = "TEST";
    uint8 public decimals = 18;
    uint256 public totalSupply;
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Mint(address indexed to, uint256 value);
    event Burn(address indexed from, uint256 value);
    
    constructor(uint256 _initialSupply) {
        totalSupply = _initialSupply;
        balances[msg.sender] = _initialSupply;
        emit Transfer(address(0), msg.sender, _initialSupply);
    }
    
    function balanceOf(address _account) public view returns (uint256) {
        return balances[_account];
    }
    
    function transfer(address _to, uint256 _amount) public returns (bool) {
        require(balances[msg.sender] >= _amount, "Insufficient balance");
        require(_to != address(0), "Cannot transfer to zero address");
        
        balances[msg.sender] -= _amount;
        balances[_to] += _amount;
        
        emit Transfer(msg.sender, _to, _amount);
        return true;
    }
    
    function approve(address _spender, uint256 _amount) public returns (bool) {
        allowances[msg.sender][_spender] = _amount;
        emit Approval(msg.sender, _spender, _amount);
        return true;
    }
    
    function transferFrom(address _from, address _to, uint256 _amount) public returns (bool) {
        require(balances[_from] >= _amount, "Insufficient balance");
        require(allowances[_from][msg.sender] >= _amount, "Insufficient allowance");
        require(_to != address(0), "Cannot transfer to zero address");
        
        balances[_from] -= _amount;
        balances[_to] += _amount;
        allowances[_from][msg.sender] -= _amount;
        
        emit Transfer(_from, _to, _amount);
        return true;
    }
    
    function mint(address _to, uint256 _amount) public {
        require(_to != address(0), "Cannot mint to zero address");
        
        totalSupply += _amount;
        balances[_to] += _amount;
        
        emit Mint(_to, _amount);
        emit Transfer(address(0), _to, _amount);
    }
    
    function burn(uint256 _amount) public {
        require(balances[msg.sender] >= _amount, "Insufficient balance");
        
        balances[msg.sender] -= _amount;
        totalSupply -= _amount;
        
        emit Burn(msg.sender, _amount);
        emit Transfer(msg.sender, address(0), _amount);
    }
}