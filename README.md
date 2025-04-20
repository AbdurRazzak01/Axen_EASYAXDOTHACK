# Axen Vault & Botaxen 🚀

A complete decentralized finance (DeFi) platform combining:
- A smart contract vault
- Telegram bot interface
- Deposit/Withdraw WebApp
- Real-time crypto pricing
- Cross-chain messaging via XCM

> Designed for Westend Asset Hub, Sepolia, and future real-world parachain deployment.

---

## 🌟 Project Structure

| Folder | Description |
|:---|:---|
| `/bot/` | Telegram bot (Node.js) |
| `/vault_contract/` | Solidity smart contract for Axen Vault |
| `/webapp/` | Frontend (Deposit / Withdraw) deployed on Netlify |

---

## 🚀 Live Links

| Part | Link |
|:---|:---|
| Live Website | [https://botaxen.netlify.app](https://botaxen.netlify.app) |
| Deposit Page | [https://botaxen.netlify.app/deposit.html](https://botaxen.netlify.app/deposit.html) |
| Withdraw Page | [https://botaxen.netlify.app/withdraw.html](https://botaxen.netlify.app/withdraw.html) |
| Telegram Bot | [Link your Telegram Bot here] |

---

## 🛠️ Smart Contract: AxenVault

**Features:**
- Secure deposits (ETH or DOT)
- Owner-controlled withdrawals
- Cross-chain messaging with XCM
- Individual user balance tracking
- Event logging for all major operations
- Upgradeable-friendly architecture
- Receives ETH via `receive()` and `fallback()` functions

**Contract Deployment:**
- Solidity ^0.8.20
- Tested and ready for parachain environments
- XCM Interface Compatible

---

## 🤖 Telegram Bot Features

- `/start` → Welcome message + Main Menu
- 💰 Deposit → Opens WebApp inside Telegram
- 🏦 Withdraw → Opens WebApp inside Telegram
- 📈 See Price → Shows real-time DOT price + mini chart
- 🧠 Subscribe to Strategy → Auto-sends strategy notifications every 30s
- Clean UX with permanent reply buttons under typing bar
- Connected to Web3 vault smart contract
- Built using Node.js & Telegram Bot API

---

## 🌐 WebApp Frontend

- Fully mobile-optimized Deposit and Withdraw interfaces
- Ethers.js based Web3 wallet connection (MetaMask)
- Live on Netlify
- Telegram WebApp ready

---

## ⚙️ Technology Stack

| Tech | Use |
|:---|:---|
| Solidity | Smart Contract |
| Node.js | Telegram Bot |
| Ethers.js | Web3 Connection |
| Netlify | WebApp Hosting |
| CoinGecko API | Real-time crypto price |
| Chart API | Price trend chart generation |

---

## 📦 Install & Run Locally

Clone the repo:

```bash
git clone https://github.com/AbdurRazzak01/Axen_EASYAXDOTHACK.git
cd Axen_EASYAXDOTHACK
