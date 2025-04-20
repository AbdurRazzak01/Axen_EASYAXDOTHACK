const { ethers } = require("ethers");
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("⏳ Starting deployment...");

    const provider = new ethers.JsonRpcProvider('https://westend-asset-hub-eth-rpc.polkadot.io');

    const privateKey = '7a1bfdc5e0389137aa3cf3c7ddbad9d6cb83abd2939e811ccf1e020ea8b8ca24';
    const wallet = new ethers.Wallet(privateKey, provider);

    const contractJsonPath = path.join(__dirname, "../build/contracts/Vault.json");
    const contractJson = JSON.parse(fs.readFileSync(contractJsonPath, "utf8"));
    const abi = contractJson.abi;
    const bytecode = contractJson.bytecode;

    const factory = new ethers.ContractFactory(abi, bytecode, wallet);

    console.log("⏳ Deploying Vault contract...");

    // ❗ DEPLOY WITHOUT constructor arguments
    const contract = await factory.deploy();  

    await contract.deploymentTransaction().wait();
    const deployedAddress = await contract.getAddress();

    console.log("✅ Vault contract deployed at:", deployedAddress);

    // ✅ Now call initialize()
    console.log("⏳ Initializing contract...");

    const xcmContractAddress = "0x0000000000000000000000000000000000000000"; // <--- change if needed
    const tx = await contract.initialize(wallet.address, xcmContractAddress);
    await tx.wait();

    console.log("✅ Vault contract initialized successfully!");
}

main().catch((error) => {
    console.error("❌ Deployment error:", error);
    process.exit(1);
});


