# OracleDrill contracts

Administrator-controlled mock report store, deliberately unsafe consumer, and a guarded Soroban consumer.

OracleDrill is an open-source failure-testing workbench for Stellar oracle consumers. It compares a deliberately unsafe reader against an explicit guarded policy. Included scenarios cover freshness boundaries, future timestamps, wrong feeds, unexpected decimals, non-positive prices, divergent references, stale references and malformed JSON values.

## Integrated setup

```sh
mkdir oracledrill && cd oracledrill
git clone https://github.com/oracledrill-labs/oracledrill-contracts.git
git clone https://github.com/oracledrill-labs/oracledrill-scenario-engine.git
git clone https://github.com/oracledrill-labs/oracledrill-backend.git
git clone https://github.com/oracledrill-labs/oracledrill-frontend.git
git clone https://github.com/oracledrill-labs/oracledrill-fixtures.git
node --test oracledrill-*/test.mjs
cargo test --locked --manifest-path oracledrill-contracts/Cargo.toml
node oracledrill-backend/server.mjs
# Another terminal:
node oracledrill-frontend/server.mjs
```

Open http://127.0.0.1:4370, load the failure corpus, run the drill and export JSON. Clone all repositories as siblings with their original names because components import one another. Node 24+, Rust stable. No npm packages are needed for the local scenario UI/API; the optional testnet scripts use pinned Stellar SDK 17.3.0 (`npm ci` in backend).

## Guard policy

The policy fixes feed, decimals, maximum age (1–86400 seconds), future skew (0–300 seconds), and maximum reference deviation (0–10000 basis points). Age and future-skew boundaries are inclusive. Deviation is measured relative to the reference answer; products use checked i128 arithmetic, rejecting overflow. A reference can be required; if supplied it must also pass feed, decimal, positivity and freshness checks. Reference independence and authenticity are not verified.

No silent decimal rescaling is performed. Price answers are signed i128 decimal strings in JSON; timestamps are uint64 strings in whole seconds. HTTP imports are limited to 1 MiB / 100 scenarios; history is in memory (latest 100 runs). Export reports before restarting. Input hashes identify the exact JSON serialization.

## Honest scope

This is an early testing MVP. Reports are synthetic; the deployed contract stores administrator-controlled MOCK prices. It is not a Chainlink adapter, does not verify provider signatures, does not supply trading prices, and does not certify production safety. Mock reports are authenticated only as administrator submissions. The reference is caller-supplied. Unsafe reads are intentionally unsafe and exist solely for the demonstration. No user funds, token transfers, wallet connection or mainnet transactions.

The JavaScript model additionally rejects malformed JSON that cannot be encoded into the contract's typed arguments. Rust tests cover contract policy behavior and authorization. See TESTNET.md for the precise distinction between confirmed transactions and RPC simulations.

## Contribute

See CONTRIBUTING.md, ROADMAP.md and SECURITY.md. All five repositories form one project. MIT licensed.

## Context

[Stellar oracle integration guide](https://stellar.org/blog/developers/ways-to-read-chainlink-prices-on-stellar-data-feeds-and-data-streams). The project is independent of oracle providers and makes no endorsement or affiliation claim.
