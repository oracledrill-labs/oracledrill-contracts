# Testnet evidence

The backend repository contains `testnet-evidence.json` after the controlled pilot. It records the contract ID, optimized Wasm SHA-256, synthetic mock reports and public transaction hashes. Rejected guarded calls are RPC simulations against the deployed contract; they are not failed transactions included in a ledger. The stale unsafe read and fresh guarded read are successful confirmed transactions. Price values are invented fixture data, not observations from a real oracle provider. No mainnet deployment is included.

Contract build:
```sh
stellar contract build --manifest-path oracledrill-contracts/Cargo.toml
```
Use a Stellar CLI compatible with soroban-sdk 29.0.1. The pilot build used CLI 28.1.0; the SDK build support was verified by compilation and testnet execution. Follow the current official CLI/SDK compatibility guidance when reproducing. No private keys are included in the evidence.

Read-only verification (RPC transaction retention is finite):
```sh
cd oracledrill-backend
npm ci
node verify-testnet.mjs
node replay-testnet.mjs
```

To submit a new disposable testnet pilot, build the Wasm first and explicitly set the paths below. The private pilot directory must stay outside your Git checkout and must not be uploaded. The script creates its own faucet-funded identity; it does not read a production key.
```sh
ORACLEDRILL_ALLOW_TESTNET=1 ORACLEDRILL_PILOT_DIR=/absolute/private/pilot ORACLEDRILL_WASM=/absolute/path/oracledrill_contracts.wasm node pilot-testnet.mjs
```
Use a fresh private pilot directory to execute all cases again. A resumed directory retains already confirmed steps. The script writes public receipts into testnet-evidence.json.
