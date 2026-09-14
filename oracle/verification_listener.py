"""
verification_listener.py
Polls the carbon_registry contract for pending projects, and pushes
monitoring-data hashes into the backend once satellite verification
completes. Runs on a loop (default every 6 hours).
"""
import os
import time
import requests

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:3001/api/v1")
ORACLE_SECRET = os.environ.get("ORACLE_SHARED_SECRET", "dev-oracle-secret")
POLL_INTERVAL_SECONDS = int(os.environ.get("VERIFICATION_POLL_SECONDS", 6 * 60 * 60))


def fetch_pending_projects():
    resp = requests.get(f"{BACKEND_URL}/projects", params={"status": "PENDING"})
    resp.raise_for_status()
    return resp.json()


def submit_monitoring_data(project_on_chain_id: str, data_hash: str):
    resp = requests.post(
        f"{BACKEND_URL}/oracle/monitoring",
        headers={"x-oracle-secret": ORACLE_SECRET},
        json={"projectOnChainId": project_on_chain_id, "dataHash": data_hash},
    )
    resp.raise_for_status()
    return resp.json()


def run_once():
    print("[INFO] Checking for pending projects...")
    projects = fetch_pending_projects()
    print(f"[INFO] Found {len(projects)} pending project(s)")
    for project in projects:
        # Placeholder: in production this calls satellite_monitor's
        # analysis pipeline to produce a real IPFS CID / data hash.
        fake_hash = f"ipfs-placeholder-{project['onChainId']}"
        submit_monitoring_data(str(project["onChainId"]), fake_hash)
        print(f"[INFO] Submitted monitoring data for project {project['onChainId']}")


def main():
    print("[INFO] Verification listener started")
    print("[INFO] Listening for new verification requests on blockchain...")
    while True:
        try:
            run_once()
        except Exception as exc:  # noqa: BLE001
            print(f"[ERROR] verification_listener failed: {exc}")
        print(f"[INFO] Next check in: {POLL_INTERVAL_SECONDS // 3600} hours")
        time.sleep(POLL_INTERVAL_SECONDS)


if __name__ == "__main__":
    main()
