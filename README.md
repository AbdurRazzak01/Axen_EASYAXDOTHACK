<p align="center">
  <img src="assets/assets" alt="Axen Logo" width="250"/>
</p>


# 🧠 Axen Strategy Mirror Pallet

## 📖 Overview

The **Axen Strategy Mirror Pallet** is a custom-built Substrate module designed to track top-performing traders across parachains, analyze their strategies, and automatically mirror profitable actions into the Axen Vault smart contract deployed on **Asset Hub**.

This system **democratizes DeFi alpha** by making elite trading strategies available to everyday users, while maintaining decentralization, security, and **cross-chain compatibility via XCM**.

> Built specifically to leverage the capabilities of the **Polkadot Asset Hub** for scalable smart contracts and XCM messaging.

---

## ⚙️ Key Innovations

| Feature | Innovation |
|:---|:---|
| ✅ Use of **Asset Hub** | Smart contracts deployed natively on Asset Hub for stability and security |
| ✅ **Cross-Chain XCM Messaging** | Vault can send/receive cross-chain asset transfers |
| ✅ **Custom Substrate Pallet** | Axen Strategy Pallet mirrors top traders automatically |
| ✅ **Telegram Bot Web3 UX** | Fully WebApp-embedded UX inside Telegram, no browser switching |
| ✅ **Live Crypto Pricing** | Real-time CoinGecko price charts integrated inside Telegram |
| ✅ **Fully Decentralized** | Strategy logic stored on-chain, transparent and upgradeable |

---

## 🛠️ Architecture Overview

| Layer | Component | Responsibility |
|:---|:---|:---|
| **Asset Hub Smart Contract** | Axen Vault | Handle deposits, withdrawals, cross-chain transfers |
| **Custom Substrate Pallet** | Strategy Mirror | Track, rank, and mirror top traders across chains |
| **Off-chain Worker** | Oracle Fetcher | Update trader performance and yield metrics |
| **Telegram Bot** | Botaxen Bot | User-friendly WebApp UI, strategy notifications |
| **Frontend WebApp** | Deposit/Withdraw Portal | MetaMask integration with Asset Hub vault |

---

## 🔄 Strategy Mirroring Flow

```mermaid
flowchart TD
    A[Track Top Traders on Chains] --> B[Analyze Yield / Deposit / Withdraw Trends]
    B --> C[Identify Winning Strategies]
    C --> D[Update AxenVault Contract (Asset Hub)]
    D --> E[Mirror Deposits/Withdrawals Automatically]
    E --> F[Notify Users via Telegram Bot]
```

---

## 📂 Axen Pallet Components

| Module | Description |
|:---|:---|
| `TopTradersStorage` | Store top performing addresses |
| `StrategyAnalyzer` | Analyze yield, ROI, volatility |
| `MirrorExecutor` | Execute cross-chain actions via XCM |
| `OffChainWorker` | Secure fetching of yield statistics |
| `Events` | Log all strategy changes for transparency |

---

## 🌉 Supported Chains (Cross-Chain Strategy)

| Chain | Mirror Activities |
|:---|:---|
| Westend Asset Hub | DOT staking, lending vaults |
| Moonbeam | LP pools, DeFi farms |
| Astar | Multi-asset DeFi strategies |
| Acala | Stablecoin and liquidity pools |

---

## 🎯 Why Axen is Different

| Benefit | Innovation |
|:---|:---|
| 🚀 Built Natively on **Asset Hub** | Ensures secure, scalable smart contract execution |
| 🔄 True Cross-Chain via XCM | Real parachain-to-parachain asset movement |
| 🤖 Automated DeFi Strategies | AI/logic driven vault mirroring |
| 💬 1-Click Web3 UX inside Telegram | Deposit/Withdraw without leaving Telegram |
| 🛡️ On-Chain Transparency | Strategy logic visible and auditable |
| 🌍 Multichain Future Ready | Designed to expand across Polkadot ecosystem |

---

## 📈 Future Enhancements

- **AI-Powered Trader Selection:** Machine learning for best strategy ranking
- **Risk Profiles:** Users choose Aggressive / Balanced / Safe mirroring modes
- **Auto-Yield Optimizer:** Dynamically rebalance vaults based on highest ROI
- **NFT Vault Badges:** Users get NFTs based on vault loyalty and performance

---

# 📦 Summary

✅ **Built on Asset Hub** for rock-solid smart contract reliability  
✅ **Cross-chain vault operations via XCM**  
✅ **Decentralized strategy tracking and mirroring**  
✅ **User-friendly Web3 experience inside Telegram**  
✅ **True multichain, multi-strategy DeFi ecosystem**

> Axen: **The Bot That Makes Money For You.**

---

# 📜 License

This project is licensed under the terms of the **Apache License 2.0**.

```
Apache License 2.0
Copyright (c) 2024 Abdur Razzak
```

See [LICENSE](LICENSE) for details.

---

# 📢 Notes

- Smart contracts are verified and deployed on Asset Hub.
- Off-chain workers guarantee reliable and up-to-date strategy metrics.
- Axen Vault is secured with OpenZeppelin standards and ReentrancyGuards.
- Strategies are mirrored only from verified, whitelisted assets and vaults.

---

# 📸 Visual System Diagram (Optional)

```mermaid
flowchart TD
    U[Users] -->|Deposit| V[Axen Vault (Asset Hub)]
    V -->|Mirror Actions| P[Axen Strategy Pallet (Substrate)]
    P -->|Cross-chain Tracking| C[Other Chains (Moonbeam, Astar, Acala)]
    P -->|Notifications| T[Telegram Bot + WebApp UX]
```
