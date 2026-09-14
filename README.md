# CarbonLedger

> Verified carbon credits. Permanent retirement. Full provenance.

A decentralized carbon credit marketplace on Stellar. Carbon projects mint
tokenized credits (RWAs) on Soroban, corporations buy and retire them
on-chain, and every credit carries a full, publicly auditable provenance
trail from issuance to retirement.

This is a from-scratch rebuild that follows the same architecture, contract
design, and lifecycle logic as the original CarbonLedger project, with a
couple of deliberate scope cuts (see below) to keep the codebase small.



## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    NEXT.JS 14 FRONTEND                       │
│   Public Audit │ Marketplace │ Dashboard                     │
└───────────────────────────┬──────────────────────────────────┘
                            │  @stellar/stellar-sdk, freighter-api
┌───────────────────────────▼──────────────────────────────────┐
│                  SOROBAN CONTRACTS (Rust)                    │
│  carbon_registry │ carbon_credit │ carbon_marketplace        │
│  carbon_oracle                                               │
└───────────────────────────┬──────────────────────────────────┘
                            │  py-stellar-base
┌───────────────────────────▼──────────────────────────────────┐
│            ORACLE / VERIFICATION BRIDGE (Python)             │
│  verification_listener │ price_oracle │ satellite_monitor    │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│          OFF-CHAIN LAYER (PostgreSQL)                        │
│  Projects │ Credit batches │ Listings │ Retirements           │
└──────────────────────────────────────────────────────────────┘
```

## Project structure

```
carbonledger/
├── contracts/          # Soroban smart contracts (Rust)
│   ├── carbon_registry/
│   ├── carbon_credit/
│   ├── carbon_marketplace/   # simplified: no bulk purchase / DEX
│   └── carbon_oracle/
├── backend/             # NestJS REST API
│   └── prisma/schema.prisma
├── frontend/            # Next.js 14 App Router
├── oracle/              # Python oracle bridge
├── scripts/             # setup / test / deploy helper scripts
├── docker-compose.yml
└── .env.example
```

## Getting started

### 1. Prerequisites

- Node.js 18+, Rust 1.74+ (with `wasm32-unknown-unknown` target), Python 3.10+,
  PostgreSQL 14+, Redis 6+, Stellar CLI

```bash
./scripts/verify-setup.sh
```

### 2. Configure

```bash
cp .env.example .env
# edit .env: DATABASE_URL, JWT_SECRET, etc.
```

### 3. Build & test contracts

```bash
cd contracts
cargo build --target wasm32-unknown-unknown --release
cargo test
```

### 4. Deploy contracts to testnet

```bash
stellar keys generate deployer --network testnet --fund
./scripts/deploy-contracts.sh deployer
# copy the printed contract IDs into .env
```

### 5. Run everything with Docker

```bash
docker-compose up --build
```

Or run each service manually — see `backend/`, `frontend/`, and `oracle/`
for their own `npm`/`pip` install + run steps.

### 6. Run all tests

```bash
./scripts/test-all.sh
```

## Smart contracts

| Contract | Purpose |
|---|---|
| `carbon_registry` | Project registration, verification, suspension |
| `carbon_credit` | Mint / transfer / permanently retire credits with unique serial numbers |
| `carbon_marketplace` | List, delist, and purchase credits (single-listing) |
| `carbon_oracle` | Monitoring-data freshness and benchmark price feeds |

## License

MIT
# CarbonLedger
