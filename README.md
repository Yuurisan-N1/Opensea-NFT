<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=OpenSea%20Mint%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=NFT%20Mint%20Sniper%20%7C%20Multi-Chain%20%7C%20Parallel%20Fanout%20%7C%20Pre-signed%20TX&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Fetch+Drop+Stages+%7C+OpenSea+API+%26+On-chain+Fallback;Select+Stage+%7C+Interactive+Menu+with+Prices+%26+Limits;Pre-sign+TX+2s+Before+Mint+Window+Opens;Fire+Parallel+RPC+Fanout+%7C+Fastest+Endpoint+Wins;Monitor+Inclusion+%7C+WebSocket+%2B+Receipt+Confirm;Multi-Chain+%7C+Ethereum+%2F+Base+%2F+Robinhood+%2F+Arc+USDC"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-OpenSea%20SeaDrop-111111"/>
  <img alt="chains" src="https://img.shields.io/badge/Chains-Ethereum%20%7C%20Base%20%7C%20Robinhood%20%7C%20Arc-111111"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>OpenSea Mint Bot</b> is a multi-chain NFT mint sniper for OpenSea SeaDrop contracts.<br/>
  It fetches all drop stages for a target contract via the OpenSea API with an on-chain fallback, shows an interactive stage menu with prices and per-wallet limits, pre-signs the mint transaction 2 seconds before the window opens, fires the signed transaction to multiple RPC endpoints in parallel, and monitors inclusion via WebSocket and receipt polling, all across Ethereum, Base, Robinhood, and Arc USDC chains simultaneously.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

<hr/>

<p align="center">
  <img src="image/image.png" alt="preview" width="100%"/>
</p>

<hr/>

## Table of Contents

- [Requirements](#requirements)
- [Installation](#installation)
- [Configuration](#configuration)
- [Running the Bot](#running-the-bot)
- [How It Works](#how-it-works)
- [Features](#features)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Rust `1.70+` (includes `cargo`) -- only needed if building from source
- Git
- An EVM wallet private key with enough native balance to cover the mint price and gas

---

## Installation

### Install Rust

**Linux / macOS:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows:**

Download and run the installer from https://rustup.rs, then restart your terminal.

**Termux (Android):**

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Then inside Ubuntu:

```bash
apt update && apt install -y curl git build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Clone the Repository

```bash
git clone https://github.com/Yuurisan-N1/Opensea-NFT.git
cd Opensea-NFT
```

---

## Configuration

### 1. Private Key (.env)

Copy `.env.example` to `.env` and fill in your EVM wallet private key:

```env
PRIVATEKEY=0xyourprivatekeyhere
```

The wallet address is derived automatically from the private key and displayed at startup. Ensure the wallet has enough native token balance on each chain you want to mint on to cover the mint price plus gas.

### 2. Proxy (proxy.txt)

Fill `proxy.txt` with one proxy (optional, leave empty to run without proxy):

```
host:port
host:port:user:pass
http://user:pass@host:port
socks5://user:pass@host:port
```

### 3. Bot Settings (config.json)

| Setting | Description |
|---|---|
| `settings.quantity` | Number of NFTs to mint per transaction |
| `settings.max_price_native` | Global maximum price in native token across all chains |
| `settings.max_price_per_chain.ethereum` | Maximum price on Ethereum in ETH |
| `settings.max_price_per_chain.base` | Maximum price on Base in ETH |
| `settings.max_price_per_chain.robinhood` | Maximum price on Robinhood chain |
| `settings.max_price_per_chain.arc_usdc` | Maximum price on Arc USDC chain |
| `settings.gas.priority_gwei` | EIP-1559 priority fee tip in Gwei |
| `settings.gas.max_gwei` | EIP-1559 max fee cap in Gwei |
| `settings.rpc_fanout` | Number of RPC endpoints to broadcast to in parallel |
| `settings.fire_window_ms` | Total broadcast retry window in milliseconds |
| `settings.fire_interval_ms` | Delay between each broadcast retry in milliseconds |

---

## Running the Bot

### Using run.sh (Linux / macOS / Termux)

Make it executable first:

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh direct    # build and run in foreground (default)
./run.sh nohup     # build and run in background, logs saved to opensea-mint-bot.log
./run.sh screen    # build and run in a detached screen session
./run.sh tmux      # build and run in a detached tmux session
./run.sh logs      # tail the log file
./run.sh stop      # stop the running bot process
```

### Using make (Linux / macOS)

```bash
make release    # optimized release build
make start      # build release and run immediately
make run        # build debug and run
make check      # check for errors without building
make fmt        # format the code
make clean      # remove all build artifacts
make size       # show release binary size
```

### Manual cargo build

```bash
cargo build --release
```

Then run:

```bash
# Linux / macOS / Termux
./target/release/opensea-mint-bot

# Windows
.\target\release\opensea-mint-bot.exe
```

---

## Download Prebuilt Binary

Prebuilt binaries are automatically compiled on every push to main via GitHub Actions across 7 targets.

Download the latest binaries from the Actions page:
https://github.com/Yuurisan-N1/Opensea-NFT/actions/workflows/build.yml

Open the latest successful run and scroll to the Artifacts section. All binaries are retained for 90 days.

| Artifact | Platform |
|---|---|
| `opensea-mint-bot-linux-x86_64` | Linux x86_64 |
| `opensea-mint-bot-linux-aarch64` | Linux ARM64 |
| `opensea-mint-bot-linux-armv7` | Linux ARMv7 |
| `opensea-mint-bot-windows-x86_64` | Windows x86_64 |
| `opensea-mint-bot-macos-aarch64` | macOS Apple Silicon |
| `opensea-mint-bot-android-aarch64` | Android ARM64 (Termux) |
| `opensea-mint-bot-android-armv7` | Android ARMv7 (Termux) |

**Linux / macOS / Termux after downloading:**

```bash
chmod +x opensea-mint-bot-linux-x86_64
./opensea-mint-bot-linux-x86_64
```

**Windows:**

```bash
.\opensea-mint-bot-windows-x86_64.exe
```

---

## How It Works

1. Enter the target NFT contract address when prompted
2. The bot fetches all drop stages from the OpenSea API for every supported chain simultaneously. If the API returns no slug for a chain, it falls back to querying the SeaDrop contract on-chain directly to find the public drop stage
3. All available stages across all chains are listed in a numbered menu showing the stage name, start and end times, price in native token, and per-wallet limit
4. You pick one or more stage numbers to arm
5. The bot connects to each chain, synchronizes its clock with the server, and waits. During the wait, the transaction is pre-signed with the current nonce, gas estimate, and EIP-1559 fee parameters 2 seconds before the stage opens
6. At the exact open timestamp, the bot broadcasts the pre-signed transaction to multiple RPC endpoints in parallel using up to `rpc_fanout` concurrent broadcasts. If the mint window is not yet active, it retries at `fire_interval_ms` intervals until `fire_window_ms` expires or a success is returned
7. After a successful broadcast, the bot monitors inclusion via WebSocket pending transaction subscription and polls for the receipt. The block number is logged on confirmation, or a revert is reported if the transaction failed on-chain

---

## Features

### Multi-Chain Support
Ethereum, Base, Robinhood, and Arc USDC chains are all supported. The bot arms all applicable chains for the target contract simultaneously. Per-chain price limits prevent accidental overpayment on any individual chain.

### Stage Discovery
Drop stages are fetched from the OpenSea API using the collection slug. If the API does not return a slug for a chain, the bot falls back to calling `getPublicDrop` on the SeaDrop contract directly to find any open public stage. Both the OpenSea API stages and the on-chain public stage are shown together in the stage menu.

### Interactive Stage Menu
All available and upcoming stages across all chains are listed with their label, start and end time in a human-readable relative format (e.g. "in 3 hours"), price in native token, and per-wallet mint limit. You select which stages to arm by number before the bot begins waiting.

### Pre-signed Transaction
The bot builds and signs the full mint transaction 2 seconds before the stage opens, using the live nonce, gas estimate, and EIP-1559 fee parameters at that moment. The signed raw transaction is held in memory ready to broadcast the instant the window opens.

### Parallel RPC Fanout
At the open timestamp, the pre-signed transaction is broadcast to up to `rpc_fanout` RPC endpoints simultaneously using concurrent async tasks. The first endpoint to accept the transaction wins and the result is returned immediately. This maximizes the chance of landing the transaction in the opening block.

### Retry Broadcast Window
If all endpoints reject the transaction because the drop window is not yet active, the bot retries the entire fanout broadcast every `fire_interval_ms` milliseconds until either a success is received or the `fire_window_ms` window expires.

### WebSocket Inclusion Monitor
After a successful broadcast, the bot subscribes to the chain's WebSocket endpoint to watch for the pending transaction and waits up to 45 seconds for it to appear in the mempool. It then polls the RPC for the transaction receipt and confirms the block number or reports a revert.

### Price and Balance Guard
Before arming any stage, the bot checks the mint price against the configured per-chain and global price limits and skips any stage that exceeds them. It also checks the wallet balance against the required amount including quantity and rejects stages the wallet cannot afford.

### Clock Synchronization
The bot synchronizes its local clock against the server before waiting for any stage to ensure the pre-sign and fire timestamps are accurate regardless of local clock drift.

### Gas Configuration
Priority fee and max fee cap are configurable per run in `config.json`. Gas is estimated live from the RPC at pre-sign time and padded with a fallback of 260,000 gas units if estimation fails.

---

## File Structure

```text
Opensea-NFT/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 7-target matrix build, artifacts retained 90 days
├── src/
│   ├── main.rs                          # Entry point and Ctrl+C handler
│   ├── lib.rs                           # Library root
│   ├── core/
│   │   ├── config.rs                    # Config loading and settings structs
│   │   ├── error.rs                     # Error types
│   │   └── types.rs                     # Shared types
│   ├── crypto/
│   │   ├── signer.rs                    # EIP-1559 transaction pre-signing
│   │   └── wallet.rs                    # Private key loading, balance check, address display
│   ├── engine/
│   │   ├── orchestrator.rs              # Main flow: chain prep, stage menu, schedule, fire
│   │   └── sniper.rs                    # Parallel RPC fanout broadcaster
│   ├── network/
│   │   ├── clock.rs                     # Server clock synchronization
│   │   ├── nonce.rs                     # Nonce fetching
│   │   ├── rpc.rs                       # RPC client, gas estimate, fee params, receipt polling
│   │   └── ws.rs                        # WebSocket inclusion monitor
│   ├── protocols/
│   │   └── opensea/
│   │       ├── api.rs                   # OpenSea API: collection slug, drop stages
│   │       ├── mod.rs                   # Chain registry (Ethereum, Base, Robinhood, Arc USDC)
│   │       ├── seadrop.rs               # SeaDrop ABI encoding and decoding
│   │       └── stage.rs                 # Stage window model, state classification, menu builder
│   └── ui/
│       ├── banner.rs                    # ASCII banner
│       ├── logger.rs                    # Colored log output, countdown, time formatter
│       ├── mod.rs
│       └── prompt.rs                    # Interactive prompts: contract address, stage picker
├── assets/
│   └── icon.ico                         # Windows binary icon
├── image/
│   └── image.png                        # Preview image shown in README
├── scripts/
│   └── package.sh                       # Package script for release distribution
├── Cargo.toml                           # Project manifest and dependencies
├── Cargo.lock
├── build.rs                             # Build script (embeds icon into Windows binary)
├── Makefile                             # make targets: build, run, release, start, clean, size
├── run.sh                               # Run helper: direct, nohup, screen, tmux, logs, stop
├── config.json                          # Quantity, price limits, gas, fanout, fire window
├── proxy.txt                            # Proxy (optional)
└── .env.example                         # Private key template
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>