// lib/stellar.ts
// Thin helpers around Freighter (wallet) and the backend API. Kept
// framework-agnostic so both server and client components can import it.

import freighterApi from '@stellar/freighter-api';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001/api/v1';

export async function connectWallet(): Promise<string> {
  const { isConnected } = await freighterApi.isConnected();
  if (!isConnected) {
    throw new Error('Freighter wallet is not installed');
  }
  const { address } = await freighterApi.requestAccess();
  return address;
}

export async function loginWithWallet(pubKey: string): Promise<string> {
  const challengeResp = await fetch(`${API_BASE}/auth/challenge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ stellarPubKey: pubKey }),
  });
  const { nonce } = await challengeResp.json();

  const { signedMessage } = await freighterApi.signMessage(nonce, { address: pubKey });

  const verifyResp = await fetch(`${API_BASE}/auth/verify`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ stellarPubKey: pubKey, signature: signedMessage }),
  });
  const { accessToken } = await verifyResp.json();
  return accessToken;
}

export async function apiGet<T>(path: string): Promise<T> {
  const resp = await fetch(`${API_BASE}${path}`, { cache: 'no-store' });
  if (!resp.ok) throw new Error(`GET ${path} failed: ${resp.status}`);
  return resp.json();
}

export async function apiPost<T>(path: string, body: unknown, token?: string): Promise<T> {
  const resp = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`POST ${path} failed: ${resp.status}`);
  return resp.json();
}
