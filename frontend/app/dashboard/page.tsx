'use client';

import { useState } from 'react';
import { connectWallet, loginWithWallet } from '../../lib/stellar';

export default function DashboardPage() {
  const [pubKey, setPubKey] = useState<string | null>(null);
  const [status, setStatus] = useState<string>('');

  async function handleConnect() {
    try {
      setStatus('Connecting to Freighter...');
      const address = await connectWallet();
      setPubKey(address);
      setStatus('Signing in...');
      await loginWithWallet(address);
      setStatus('Connected');
    } catch (err) {
      setStatus(err instanceof Error ? err.message : 'Connection failed');
    }
  }

  return (
    <div>
      <h1>Dashboard</h1>
      {!pubKey ? (
        <button onClick={handleConnect}>Connect Freighter Wallet</button>
      ) : (
        <p>Connected as {pubKey}</p>
      )}
      <p>{status}</p>
      <p>
        From here, project developers can register projects, corporations can view purchased
        and retired credits, and verifiers can review pending projects.
      </p>
    </div>
  );
}
