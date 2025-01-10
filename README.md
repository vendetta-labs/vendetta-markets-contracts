# Vendetta Markets

Embrace the adrenaline of esports and the power of decentralized finance with Vendetta Markets, the ultimate platform where skill meets opportunity. Bet on your favorite teams, earn rewards, and experience the thrill of victory like never before.

With a transparent and efficient technical infrastructure, Vendetta Markets offers a seamless and fair betting experience for esports enthusiasts.

This repository contains the source code for the core smart contracts of Vendetta Markets. Smart contracts are meant to be compiled to `.wasm` files and uploaded to the Cosmos chains.

## How to develop

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.81
- [wasm32-unknown-unknown target](https://docs.cosmwasm.com/core/installation)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.81
rustup target add wasm32-unknown-unknown
```

### Run tests

```bash
cargo test
```
## How to deploy

### Compile contracts

```bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

### Check contracts

Check `parimutuel-market` contract:
```bash
cosmwasm-check ./target/wasm32-unknown-unknown/release/parimutuel_market.wasm
```

Check `fixed-odds-market` contract:
```bash
cosmwasm-check ./target/wasm32-unknown-unknown/release/fixed_odds_market.wasm
```

### Deploy contracts

After compiling the contracts, you can deploy them.

Deploying `parimutuel-market` contract:

```bash
neutrond tx wasm store "./target/wasm32-unknown-unknown/release/parimutuel_market.wasm" --from vendetta-markets-deployer --gas auto --gas-prices 0.009untrn --gas-adjustment 1.3 -y --chain-id=pion-1 -b sync -o json --node $NODE
```

Deploying `fixed-odds-market` contract:

```bash
neutrond tx wasm store "./target/wasm32-unknown-unknown/release/fixed_odds_market.wasm" --from vendetta-markets-deployer --gas auto --gas-prices 0.009untrn --gas-adjustment 1.3 -y --chain-id=pion-1 -b sync -o json --node $NODE
```