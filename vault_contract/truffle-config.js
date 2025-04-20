const HDWalletProvider = require('@truffle/hdwallet-provider');

module.exports = {
  networks: {
    polkadot: {
      provider: () => new HDWalletProvider(
        '7a1bfdc5e0389137aa3cf3c7ddbad9d6cb83abd2939e811ccf1e020ea8b8ca24', // Your private key
        'https://westend-asset-hub-eth-rpc.polkadot.io',                  // Westend Asset Hub RPC
        0 // Index of the account
      ),
      network_id: 420420421,  // ✅ Correct network ID
      gas: 5000000,           // ✅ Gas limit
      gasPrice: 20000000000,  // ✅ Gas price
      from: '0x3016DBeE1F9580638E2691546e8D2df1535B03be', // ✅ Your wallet address
      skipDryRun: true,       // ✅ Avoid dry-run because Asset Hub may reject it
    },
  },

  compilers: {
    solc: {
      version: '0.8.0', // ✅ Solidity version 0.8.0
      settings: {
        optimizer: {
          enabled: true,
          runs: 200,
        },
        evmVersion: "berlin", // ✅ Required: Berlin EVM for Asset Hub
      },
    },
  },
};
