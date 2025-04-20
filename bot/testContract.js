const Web3 = require('web3');
const fs = require('fs');
const path = require('path');

// Web3 setup (use a Web3 provider compatible with your contract network)
const web3 = new Web3('wss://westend-rpc.polkadot.io'); // Use your correct network RPC URL

// Contract ABI and Address
const contractABI = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../vault_contract/contracts/abi/VaultABI.json'), 'utf-8'));
const contractAddress = '0xd9145CCE52D386f254917e481eB44e9943F39138'; // Replace with your contract address

// Create contract instance
const vaultContract = new web3.eth.Contract(contractABI, contractAddress);

// Test contract interactions
async function testContract() {
  console.log('Testing contract interactions...');
  
  // Ensure we have accounts to interact with
  const accounts = await web3.eth.getAccounts();
  if (accounts.length === 0) {
    console.log('No accounts found. Please make sure your wallet is connected.');
    return;
  }
  const account = accounts[0]; // Use the first account for transactions
  console.log(`Using account: ${account}`);

  // Deposit funds (for testing)
  const depositAmount = '0.1'; // Example: 0.1 ETH (adjust as necessary)
  await depositFunds(depositAmount, account);

  // Withdraw funds (for testing)
  const withdrawAmount = '0.05'; // Example: 0.05 ETH (adjust as necessary)
  await withdrawFunds(withdrawAmount, account);
}

// Deposit function (send funds to the contract)
async function depositFunds(amount, fromAddress) {
  try {
    console.log(`Depositing ${amount} ETH from address ${fromAddress}`);
    await vaultContract.methods.deposit().send({
      from: fromAddress,
      value: web3.utils.toWei(amount, 'ether') // Ensure the amount is properly converted to Wei
    });
    console.log(`Deposited ${amount} ETH successfully.`);
  } catch (err) {
    console.error('Error depositing funds:', err.message);
  }
}

// Withdraw function (withdraw funds from the contract)
async function withdrawFunds(amount, fromAddress) {
  try {
    console.log(`Withdrawing ${amount} ETH from address ${fromAddress}`);
    await vaultContract.methods.withdraw(web3.utils.toWei(amount, 'ether')).send({
      from: fromAddress
    });
    console.log(`Withdrew ${amount} ETH successfully.`);
  } catch (err) {
    console.error('Error withdrawing funds:', err.message);
  }
}

// Run the test contract
testContract();
