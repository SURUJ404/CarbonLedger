"""
satellite_monitor.py
Lightweight Flask webhook receiver for satellite monitoring providers
(e.g. Google Earth Engine exports). Forwards verified monitoring events
to the backend's /oracle/monitoring endpoint.
"""
import os
from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:3001/api/v1")
ORACLE_SECRET = os.environ.get("ORACLE_SHARED_SECRET", "dev-oracle-secret")
PORT = int(os.environ.get("SATELLITE_WEBHOOK_PORT", 5001))


@app.route("/webhook/satellite", methods=["POST"])
def satellite_webhook():
    payload = request.get_json(force=True) or {}
    project_on_chain_id = payload.get("projectOnChainId")
    data_hash = payload.get("dataHash")

    if not project_on_chain_id or not data_hash:
        return jsonify({"error": "projectOnChainId and dataHash are required"}), 400

    resp = requests.post(
        f"{BACKEND_URL}/oracle/monitoring",
        headers={"x-oracle-secret": ORACLE_SECRET},
        json={"projectOnChainId": project_on_chain_id, "dataHash": data_hash},
    )
    resp.raise_for_status()
    return jsonify(resp.json()), 200


@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok"})


if __name__ == "__main__":
    print("[INFO] Satellite monitor started")
    print(f"[INFO] Listening for webhooks on port {PORT}")
    print("[INFO] Ready to receive satellite data from Google Earth Engine")
    app.run(host="0.0.0.0", port=PORT)
