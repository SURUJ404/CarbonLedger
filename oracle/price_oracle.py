"""
price_oracle.py
Fetches benchmark carbon credit prices per methodology/vintage and pushes
them to the backend, which relays them on-chain via carbon_oracle's
update_credit_price(). Runs on a loop (default every 12 hours).

NOTE: real Xpansiv CBL / Toucan Protocol API integration is left as a
config-driven stub (PRICE_FEED_URL) so this can run standalone in dev
without external credentials.
"""
import os
import time
import requests

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:3001/api/v1")
ORACLE_SECRET = os.environ.get("ORACLE_SHARED_SECRET", "dev-oracle-secret")
POLL_INTERVAL_SECONDS = int(os.environ.get("PRICE_POLL_SECONDS", 12 * 60 * 60))

# methodology -> vintage_year -> price_per_tonne (USDC stroops, 7 decimals)
METHODOLOGIES = [
    ("VM0007", 2024),
    ("VM0015", 2024),
    ("AR-ACM0003", 2023),
]


def fetch_benchmark_price(methodology: str, vintage_year: int) -> int:
    # Placeholder deterministic price generator standing in for a real
    # Xpansiv CBL / Toucan Protocol API call.
    base = 8_000_000  # $0.80 USDC per tonne, in stroops
    return base + (vintage_year % 10) * 100_000


def push_price(methodology: str, vintage_year: int, price_per_tonne: int):
    resp = requests.post(
        f"{BACKEND_URL}/oracle/price",
        headers={"x-oracle-secret": ORACLE_SECRET},
        json={
            "methodology": methodology,
            "vintageYear": vintage_year,
            "pricePerTonne": str(price_per_tonne),
        },
    )
    resp.raise_for_status()
    return resp.json()


def run_once():
    print("[INFO] Fetching benchmark prices from Xpansiv CBL and Toucan Protocol...")
    for methodology, vintage_year in METHODOLOGIES:
        price = fetch_benchmark_price(methodology, vintage_year)
        push_price(methodology, vintage_year, price)
        print(f"[INFO] Updated {methodology} {vintage_year}: {price} stroops/tonne")


def main():
    print("[INFO] Price oracle started")
    while True:
        try:
            run_once()
        except Exception as exc:  # noqa: BLE001
            print(f"[ERROR] price_oracle failed: {exc}")
        print(f"[INFO] Next update in: {POLL_INTERVAL_SECONDS // 3600} hours")
        time.sleep(POLL_INTERVAL_SECONDS)


if __name__ == "__main__":
    main()
