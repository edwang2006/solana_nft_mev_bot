# Solana NFT MEV Bot

## 📝 Overview
A production-grade MEV arbitrage engine on Solana, specifically targeting NFT markets like Tensor Swap. 

## 🛠 Tech Stack
- **Language**: Rust
- **Framework**: Tokio (Async Runtime)
- **Features**: 
  - Real-time WSS log ingestion.
  - Strategy engine with LRU deduplication.
  - Transaction manager with Jito-bribe support.

## 🏗 Architecture
- **Scout**: High-speed Geyser/WSS listener.
- **Brain**: Real-time signal parsing and profit calculation.
- **Executor**: Transaction signing and broadcasting via Jito bundles.

## ⚠️ Disclaimer
This project was developed for educational purposes with assistance from Gemini. 
Use at your own risk. Digital assets involve significant financial risk.
