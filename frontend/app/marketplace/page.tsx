import { apiGet } from '../../lib/stellar';

interface Listing {
  id: string;
  pricePerTonne: string;
  tonnes: string;
  batch: {
    vintageYear: number;
    project: { name: string; methodology: string };
  };
}

// Simplified marketplace: single-listing purchase only. Bulk purchase
// across multiple projects and DEX-based secondary trading were dropped
// from this build to keep the flow small.
export default async function MarketplacePage() {
  let listings: Listing[] = [];
  try {
    listings = await apiGet<Listing[]>('/marketplace/listings');
  } catch {
    listings = [];
  }

  return (
    <div>
      <h1>Marketplace</h1>
      <table>
        <thead>
          <tr>
            <th>Project</th>
            <th>Methodology</th>
            <th>Vintage</th>
            <th>Tonnes</th>
            <th>Price / tonne (USDC)</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {listings.map((l) => (
            <tr key={l.id}>
              <td>{l.batch.project.name}</td>
              <td>{l.batch.project.methodology}</td>
              <td>{l.batch.vintageYear}</td>
              <td>{l.tonnes}</td>
              <td>{(Number(l.pricePerTonne) / 1e7).toFixed(2)}</td>
              <td>
                <button>Buy</button>
              </td>
            </tr>
          ))}
          {listings.length === 0 && (
            <tr>
              <td colSpan={6}>No active listings yet.</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
