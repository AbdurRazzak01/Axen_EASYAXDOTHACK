import { ApiPromise, WsProvider } from 'https://unpkg.com/@polkadot/api@9.14.2?module';
import {
  web3Enable,
  web3Accounts,
  web3FromAddress
} from 'https://unpkg.com/@polkadot/extension-dapp@0.44.2?module';

let api;
let selectedAccount;
let injector;

const connectBtn = document.getElementById('connectBtn');
const depositBtn = document.getElementById('depositBtn');
const walletStatus = document.getElementById('walletStatus');
const amountInput = document.getElementById('amount');

// WebSocket connection to your Polkadot network
const provider = new WsProvider('wss://rpc.polkadot.io'); // Use your Polkadot node URL (like Westend RPC URL)
api = await ApiPromise.create({ provider });

connectBtn.onclick = async function connectWallet() {
  // Requesting Polkadot extension and checking if it's installed
  const extensions = await web3Enable('Axen Telegram Bot');
  if (!extensions || extensions.length === 0) {
    return alert('⚠️ No Polkadot extension found. Please install the Polkadot.js extension.');
  }

  const accounts = await web3Accounts();
  if (!accounts.length) return alert('⚠️ No accounts found in the extension.');

  selectedAccount = accounts[0];
  walletStatus.innerText = `✅ Connected: ${selectedAccount.address}`;

  injector = await web3FromAddress(selectedAccount.address);
};

depositBtn.onclick = async function sendDeposit() {
  const amount = document.getElementById('amount').value;
  if (!amount || !selectedAccount || !injector || !api) {
    return alert('⚠️ Please connect wallet and enter a valid amount.');
  }

  const value = BigInt(amount * 1e12); // Convert DOT to Planck (1 DOT = 1e12 Planck)
  const tx = api.tx.vault.deposit(value); // Replace with correct contract call

  try {
    // Sign and send the transaction using the injector from MetaMask
    const unsub = await tx.signAndSend(
      selectedAccount.address,
      { signer: injector.signer },
      (result) => {
        if (result.status.isInBlock) {
          alert(`✅ Deposit included in block: ${result.status.asInBlock}`);
          unsub();
        }
      }
    );
  } catch (err) {
    console.error('❌ Transaction failed:', err);
    alert('❌ Failed to submit transaction.');
  }
};
