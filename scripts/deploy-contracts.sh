#!/usr/bin/env bash
# Builds and deploys all 4 Soroban contracts to the configured network.
set -e

NETWORK="${STELLAR_NETWORK:-testnet}"
SOURCE="${1:-deployer}"

cd contracts
echo "Building contracts (release, wasm32)..."
cargo build --target wasm32-unknown-unknown --release

deploy() {
  local name="$1"
  echo "Deploying $name..."
  stellar contract deploy \
    --wasm "target/wasm32-unknown-unknown/release/${name}.wasm" \
    --source "$SOURCE" \
    --network "$NETWORK"
}

deploy carbon_registry
deploy carbon_credit
deploy carbon_marketplace
deploy carbon_oracle

echo "Save the printed contract IDs into your .env file."
