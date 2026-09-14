// Public audit explorer - no wallet connection required, matching the
// original's "browse full audit trail without a wallet" design.
export default function AuditPage() {
  return (
    <div>
      <h1>Public Audit Trail</h1>
      <p>Look up any serial number to see its complete history — from project registration to retirement.</p>
      <form action="/audit/lookup" method="get">
        <input name="serial" placeholder="Serial number" style={{ padding: '0.5rem', marginRight: '0.5rem' }} />
        <button type="submit">Look up</button>
      </form>
    </div>
  );
}
