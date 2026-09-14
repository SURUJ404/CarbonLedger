#!/usr/bin/env bash
# Verifies local dev environment has everything CarbonLedger needs.
set -e

check() {
  if command -v "$1" >/dev/null 2>&1; then
    echo "OK   $1 found: $($1 --version 2>&1 | head -n1)"
  else
    echo "MISSING  $1 not found"
  fi
}

echo "Checking prerequisites..."
check node
check npm
check rustc
check cargo
check python3
check psql
check redis-cli
check git

echo ""
echo "Checking for wasm32 target..."
if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
  echo "OK   wasm32-unknown-unknown installed"
else
  echo "MISSING  run: rustup target add wasm32-unknown-unknown"
fi

echo ""
echo "Checking for Stellar CLI..."
check stellar

echo ""
echo "Done. Fix any MISSING items above before continuing."
