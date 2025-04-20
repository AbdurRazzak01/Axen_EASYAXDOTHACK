// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Axen Vault - Cross-chain DeFi Vault using XCM
/// @author YOU
/// @notice Allows ETH deposits, withdrawals, and cross-chain message sending using XCM
/// @custom:security-contact dev@axen.com
interface IXcmInterface {
    function sendMessage(address destination, bytes calldata message) external;
}

contract AxenVault {
    address private _owner;
    IXcmInterface private _xcmContract;
    bool private _initialized;

    uint256 private _vaultBalance;

    mapping(address => uint256) private _userBalances;

    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event CrossChainTransaction(address indexed destination, uint256 amount);
    event Initialized(address owner, address xcmContract);

    modifier onlyOwner() {
        require(msg.sender == _owner, "AxenVault: caller is not the owner");
        _;
    }

    modifier isInitialized() {
        require(_initialized, "AxenVault: contract not initialized");
        _;
    }

    constructor() {
        _initialized = false;
    }

    /// @notice One-time contract setup (like a constructor)
    function initialize(address owner_, address xcmContract_) external {
        require(!_initialized, "AxenVault: already initialized");
        require(owner_ != address(0), "Invalid owner address");
        require(xcmContract_ != address(0), "Invalid XCM contract address");

        _owner = owner_;
        _xcmContract = IXcmInterface(xcmContract_);
        _initialized = true;

        emit Initialized(owner_, xcmContract_);
    }

    /// @notice Deposit ETH into the vault
    receive() external payable {
        deposit();
    }

    fallback() external payable {
        deposit();
    }

    function deposit() public payable isInitialized {
        require(msg.value > 0, "AxenVault: cannot deposit zero");

        _vaultBalance += msg.value;
        _userBalances[msg.sender] += msg.value;

        emit Deposited(msg.sender, msg.value);
    }

    /// @notice Withdraw ETH from the vault (owner-only)
    function withdraw(uint256 amount) external onlyOwner isInitialized {
        require(amount > 0, "AxenVault: amount must be greater than 0");
        require(amount <= _vaultBalance, "AxenVault: insufficient vault balance");

        _vaultBalance -= amount;
        payable(_owner).transfer(amount);

        emit Withdrawn(_owner, amount);
    }

    /// @notice Send cross-chain message (using XCM)
    function sendCrossChainMessage(address destination, uint256 amount) external onlyOwner isInitialized {
        require(address(_xcmContract) != address(0), "AxenVault: XCM contract not set");
        require(destination != address(0), "AxenVault: destination invalid");

        // Customize the message structure as per your XCM handler
        bytes memory message = abi.encodePacked("TRANSFER:", toAsciiString(_owner), ":", amount);
        _xcmContract.sendMessage(destination, message);

        emit CrossChainTransaction(destination, amount);
    }

    /// @notice Vault total balance
    function getBalance() external view returns (uint256) {
        return _vaultBalance;
    }

    /// @notice Get balance of individual user
    function getUserBalance(address user) external view returns (uint256) {
        return _userBalances[user];
    }

    /// @notice Returns owner address
    function getOwner() external view returns (address) {
        return _owner;
    }

    // Utility: address => string (for message formatting)
    function toAsciiString(address x) internal pure returns (string memory) {
        bytes memory s = new bytes(42);
        s[0] = "0";
        s[1] = "x";
        for (uint i = 0; i < 20; i++) {
            bytes1 b = bytes1(uint8(uint(uint160(x)) / (2**(8*(19 - i)))));
            bytes1 hi = bytes1(uint8(b) / 16);
            bytes1 lo = bytes1(uint8(b) - 16 * uint8(hi));
            s[2*i + 2] = char(hi);
            s[2*i + 3] = char(lo);
        }
        return string(s);
    }

    function char(bytes1 b) internal pure returns (bytes1 c) {
        if (uint8(b) < 10) return bytes1(uint8(b) + 48);  // 0-9
        else return bytes1(uint8(b) + 87);                // a-f
    }
}
