const Web3 = require('web3');  // Import web3
const TelegramBot = require('node-telegram-bot-api');  // Import Telegram bot API
const fs = require('fs');
const path = require('path');

// Telegram Bot Token (get this from BotFather on Telegram)
const botToken = '8152846708:AAHMIOpvbL9SWufL4HDu5KEnWdfWIrow1EE';  // Replace with your actual Telegram Bot token
const bot = new TelegramBot(botToken, { polling: true });

// Web3 and contract setup
const web3 = new Web3('https://westend-asset-hub-eth-rpc.polkadot.io'); // Westend RPC URL
const contractABI = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../vault_contract/contracts/abi/VaultABI.json'), 'utf-8'));
const contractAddress = '0xd9145CCE52D386f254917e481eB44e9943F39138'; // Your contract address
const vaultContract = new web3.eth.Contract(contractABI, contractAddress);

// Bot command: /start
bot.onText(/\/start/, (msg) => {
  const chatId = msg.chat.id;
  console.log('Received /start command');
  bot.sendMessage(chatId, 'Welcome! You can check balance or deposit/withdraw ETH. Use /balance, /deposit <amount>, or /withdraw <amount>.');
});

// Bot command: /balance
bot.onText(/\/balance/, async (msg) => {
  const chatId = msg.chat.id;
  console.log('Received /balance command');
  try {
    const balance = await getBalance();
    console.log('Fetched balance:', balance);
    bot.sendMessage(chatId, `Your contract balance is ${balance} ETH.`);
  } catch (err) {
    console.error('Error fetching balance:', err);
    bot.sendMessage(chatId, `Error fetching balance: ${err.message}`);
  }
});

// Bot command: /deposit <amount>
bot.onText(/\/deposit (\d+)/, async (msg, match) => {
  const chatId = msg.chat.id;
  const amount = match[1];
  console.log(`Received /deposit command for amount: ${amount}`);

  // Send link to deposit webpage with the amount as a query parameter
  const depositUrl = `https://botaxen.netlify.app/deposit.html?amount=${amount}`;
  bot.sendMessage(chatId, `To deposit ${amount} DOT, please visit the following page: ${depositUrl}`);
});

// Bot command: /withdraw <amount>
bot.onText(/\/withdraw (\d+)/, async (msg, match) => {
  const chatId = msg.chat.id;
  const amount = match[1];
  console.log(`Received /withdraw command for amount: ${amount}`);

  // Send link to withdraw webpage with the amount as a query parameter
  const withdrawUrl = `https://botaxen.netlify.app/withdraw.html?amount=${amount}`;
  bot.sendMessage(chatId, `To withdraw ${amount} DOT, please visit the following page: ${withdrawUrl}`);
});

// Get balance function
async function getBalance() {
  console.log('Calling getBalance() function...');
  try {
    const balance = await vaultContract.methods.getBalance().call();
    console.log('Contract balance fetched:', balance);
    return web3.utils.fromWei(balance, 'ether'); // Convert balance from Wei to DOT (or ETH, based on the contract)
  } catch (err) {
    console.error('Error fetching balance:', err);
    throw new Error('Error fetching balance');
  }
}

// Deposit function (send funds to the contract)
async function depositFunds(amount, fromAddress) {
  console.log('Starting deposit function...');
  const accounts = [fromAddress]; // Get user's MetaMask address
  try {
    console.log(`Sending deposit of ${amount} to contract from address ${accounts[0]}`);
    const receipt = await vaultContract.methods.deposit().send({
      from: accounts[0],
      value: web3.utils.toWei(amount, 'ether') // Convert amount from DOT to Wei (you can change it if using different token units)
    });
    console.log('Deposit transaction receipt:', receipt);
  } catch (error) {
    console.error('Deposit Error:', error);
    throw new Error('Deposit failed. Please try again later.');
  }
}

// Withdraw function (withdraw funds from the contract)
async function withdrawFunds(amount, fromAddress) {
  console.log('Starting withdraw function...');
  const accounts = [fromAddress]; // Get user's MetaMask address
  try {
    console.log(`Withdrawing ${amount} from contract from address ${accounts[0]}`);
    const receipt = await vaultContract.methods.withdraw(web3.utils.toWei(amount, 'ether')).send({
      from: accounts[0]
    });
    console.log('Withdraw transaction receipt:', receipt);
  } catch (error) {
    console.error('Withdraw Error:', error);
    throw new Error('Withdrawal failed. Please try again later.');
  }
}
