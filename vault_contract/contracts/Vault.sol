// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IXcmInterface {
    function sendMessage(address destination, bytes calldata message) external;
}

contract Vault {
    // @custom:dev-run-script

    uint256 private _balance;
    address private _owner;
    IXcmInterface private _xcmContract;
    bool private _initialized;

    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event CrossChainTransaction(address indexed destination, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == _owner, "Not the owner");
        _;
    }

    // Add a constructor with initialization state
    constructor() {
        // Indicating the contract is uninitialized initially
        _initialized = false;
    }

    // Initialize function that must be called after contract deployment
    function initialize(address owner_, address xcmContract_) external {
        require(!_initialized, "Already initialized");
        _owner = owner_;
        _xcmContract = IXcmInterface(xcmContract_);
        _initialized = true;
    }

    function deposit() external payable {
        _balance += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    function withdraw(uint256 amount) external onlyOwner {
        require(amount <= _balance, "Insufficient balance");
        _balance -= amount;
        payable(_owner).transfer(amount);
        emit Withdrawn(msg.sender, amount);
    }

    function sendCrossChainMessage(address destination, uint256 amount) external onlyOwner {
        require(address(_xcmContract) != address(0), "XCM contract not set");
        bytes memory message = abi.encodePacked("Transfer", amount);
        _xcmContract.sendMessage(destination, message);
        emit CrossChainTransaction(destination, amount);
    }

    function getBalance() external view returns (uint256) {
        return _balance;
    }
}
