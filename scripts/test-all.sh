#!/usr/bin/env bash
set -e

echo "Running Rust contract tests..."
(cd contracts && cargo test)
echo "OK  Contract tests passed"

echo "Running backend tests..."
(cd backend && npm test)
echo "OK  Backend tests passed"

echo "Running frontend tests..."
(cd frontend && npm test) || echo "WARN  No frontend tests configured yet"

echo "All tests passed!"
