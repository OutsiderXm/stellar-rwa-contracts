# Quickstart: Clone to Working Asset

From fresh clone to a deployed and exercised asset on testnet in ~10 minutes.

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install soroban-cli
```

## Steps

### 1. Clone and Build

```bash
git clone https://github.com/RWA-ToolKit/stellar-rwa-contracts.git
cd stellar-rwa-contracts
cargo build --release --target wasm32-unknown-unknown
```

### 2. Deploy Contracts

```bash
export SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/asset_token.wasm \
  --source-account <your-public-key>
```

Save the contract ID.

### 3. Initialize Asset

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account <your-public-key> \
  -- initialize \
  --admin <your-public-key> \
  --name "Test Asset" \
  --symbol "TEST" \
  --decimals 2 \
  --initial_supply 1000000
```

### 4. Mint and Transfer

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account <your-public-key> \
  -- mint \
  --to <recipient-public-key> \
  --amount 50000
```

Done! Asset deployed and exercised on testnet.

See `docs/` for deeper guides.
