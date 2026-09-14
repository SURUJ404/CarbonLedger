import { apiGet } from '../../../lib/stellar';

interface BatchResult {
  onChainId: string;
  vintageYear: number;
  serialStart: string;
  serialEnd: string;
  retired: boolean;
  project: { name: string; methodology: string; status: string };
  retirement?: { beneficiary: string; reason: string; certificateUrl: string; retiredAt: string };
}

export default async function AuditLookupPage({
  searchParams,
}: {
  searchParams: { serial?: string };
}) {
  const serial = searchParams.serial;
  if (!serial) {
    return <p>Enter a serial number on the previous page.</p>;
  }

  let batch: BatchResult | null = null;
  let error: string | null = null;
  try {
    batch = await apiGet<BatchResult>(`/credits/serial/${encodeURIComponent(serial)}`);
  } catch {
    error = 'No credit batch contains that serial number.';
  }

  if (error || !batch) {
    return <p>{error}</p>;
  }

  return (
    <div>
      <h1>Serial #{serial}</h1>
      <p>
        <strong>Project:</strong> {batch.project.name} ({batch.project.methodology}) —{' '}
        {batch.project.status}
      </p>
      <p>
        <strong>Vintage:</strong> {batch.vintageYear}
      </p>
      <p>
        <strong>Serial range:</strong> {batch.serialStart}–{batch.serialEnd}
      </p>
      <p>
        <strong>Status:</strong> {batch.retired ? 'Retired (permanent, irreversible)' : 'Active'}
      </p>
      {batch.retirement && (
        <div>
          <h2>Retirement Certificate</h2>
          <p>Beneficiary: {batch.retirement.beneficiary}</p>
          <p>Reason: {batch.retirement.reason}</p>
          <p>Retired at: {batch.retirement.retiredAt}</p>
          <p>
            <a href={batch.retirement.certificateUrl}>View permanent certificate →</a>
          </p>
        </div>
      )}
    </div>
  );
}
