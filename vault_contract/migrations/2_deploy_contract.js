const Vault = artifacts.require("Vault");

module.exports = function (deployer) {
  // The XCM precompiled contract address on Asset Hub (replace this with the actual address)
  const xcmAddress = "0x000000000000000000000000000000000000081A"; // Example address, replace with actual

  // Deploy the Vault contract, passing the XCM contract address to the constructor
  deployer.deploy(Vault, xcmAddress);
};
