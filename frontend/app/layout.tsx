import './globals.css';
import type { ReactNode } from 'react';

export const metadata = {
  title: 'CarbonLedger',
  description: 'Verified carbon credits. Permanent retirement. Full provenance.',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <nav style={{ padding: '1rem 2rem', borderBottom: '1px solid #e5e5e5', display: 'flex', gap: '1.5rem' }}>
          <strong>CarbonLedger</strong>
          <a href="/marketplace">Marketplace</a>
          <a href="/audit">Public Audit</a>
          <a href="/dashboard">Dashboard</a>
        </nav>
        <main style={{ padding: '2rem' }}>{children}</main>
      </body>
    </html>
  );
}
