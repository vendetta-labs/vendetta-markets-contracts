# Vendetta Markets

Embrace the adrenaline of esports and the power of decentralized finance with Vendetta Markets, the ultimate platform where skill meets opportunity. Bet on your favorite teams, earn rewards, and experience the thrill of victory like never before.

With a transparent and efficient technical infrastructure, Vendetta Markets offers a seamless and fair betting experience for esports enthusiasts.

This repository contains the source code for the core smart contracts of Vendetta Markets. Smart contracts are meant to be compiled to `.wasm` files and uploaded to the Cosmos chains.

## How to develop

### Run tests

```bash
cargo test
```
## How to deploy

### Compile contracts

```bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

### Deploy contracts

After compiling the contracts, you can deploy them.