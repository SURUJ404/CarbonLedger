#![no_std]

mod error;
pub use error::CarbonError;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

const MONITORING_FRESHNESS_LEDGERS: u64 = 365 * 17_280; // ~365 days at 5s/ledger

#[contracttype]
#[derive(Clone, Debug)]
pub struct MonitoringData {
    pub project_id: u64,
    pub submitted_ledger: u64,
    pub data_hash: String, // IPFS CID or hash of satellite monitoring payload
    pub flagged: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BenchmarkPrice {
    pub methodology: String,
    pub vintage_year: u32,
    pub price_per_tonne: i128, // USDC stroops
    pub updated_ledger: u64,
}

const MONITORING: Symbol = Symbol::short("MONITOR");
const PRICE: Symbol = Symbol::short("PRICE");
const ADMIN: Symbol = Symbol::short("ADMIN");
const ORACLE_SIGNER: Symbol = Symbol::short("SIGNER");

#[contract]
pub struct CarbonOracle;

#[contractimpl]
impl CarbonOracle {
    pub fn initialize(env: Env, admin: Address, oracle_signer: Address) {
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&ORACLE_SIGNER, &oracle_signer);
    }

    /// Verifier/oracle bridge pushes satellite monitoring data on-chain.
    pub fn submit_monitoring_data(
        env: Env,
        signer: Address,
        project_id: u64,
        data_hash: String,
    ) -> Result<(), CarbonError> {
        signer.require_auth();
        Self::require_signer(&env, &signer)?;

        let data = MonitoringData {
            project_id,
            submitted_ledger: env.ledger().sequence() as u64,
            data_hash,
            flagged: false,
        };
        env.storage()
            .persistent()
            .set(&(MONITORING, project_id), &data);
        Ok(())
    }

    /// Push a benchmark price per methodology and vintage year. Rejects
    /// updates that move price more than 15% from the last value in one
    /// step, guarding against a single bad price feed read.
    pub fn update_credit_price(
        env: Env,
        signer: Address,
        methodology: String,
        vintage_year: u32,
        price_per_tonne: i128,
    ) -> Result<(), CarbonError> {
        signer.require_auth();
        Self::require_signer(&env, &signer)?;

        if price_per_tonne <= 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }

        let key = (PRICE.clone(), methodology.clone(), vintage_year);
        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<_, BenchmarkPrice>(&key)
        {
            let diff = (price_per_tonne - existing.price_per_tonne).abs();
            let max_move = existing.price_per_tonne * 15 / 100;
            if diff > max_move {
                // Reject outlier updates rather than silently accepting them.
                return Err(CarbonError::PriceNotSet);
            }
        }

        let price = BenchmarkPrice {
            methodology,
            vintage_year,
            price_per_tonne,
            updated_ledger: env.ledger().sequence() as u64,
        };
        env.storage().persistent().set(&key, &price);
        Ok(())
    }

    /// Verifier flags a project for investigation (e.g. suspicious satellite data).
    pub fn flag_project(env: Env, signer: Address, project_id: u64) -> Result<(), CarbonError> {
        signer.require_auth();
        Self::require_signer(&env, &signer)?;

        let mut data: MonitoringData = env
            .storage()
            .persistent()
            .get(&(MONITORING, project_id))
            .ok_or(CarbonError::ProjectNotFound)?;
        data.flagged = true;
        env.storage()
            .persistent()
            .set(&(MONITORING, project_id), &data);
        Ok(())
    }

    /// Returns false if no monitoring data was submitted in the last 365 days.
    pub fn is_monitoring_current(env: Env, project_id: u64) -> bool {
        match env
            .storage()
            .persistent()
            .get::<_, MonitoringData>(&(MONITORING, project_id))
        {
            Some(data) => {
                let now = env.ledger().sequence() as u64;
                now.saturating_sub(data.submitted_ledger) <= MONITORING_FRESHNESS_LEDGERS
            }
            None => false,
        }
    }

    pub fn get_benchmark_price(
        env: Env,
        methodology: String,
        vintage_year: u32,
    ) -> Result<BenchmarkPrice, CarbonError> {
        env.storage()
            .persistent()
            .get(&(PRICE, methodology, vintage_year))
            .ok_or(CarbonError::PriceNotSet)
    }

    fn require_signer(env: &Env, signer: &Address) -> Result<(), CarbonError> {
        let authorized: Address = env
            .storage()
            .instance()
            .get(&ORACLE_SIGNER)
            .ok_or(CarbonError::UnauthorizedOracle)?;
        if &authorized != signer {
            return Err(CarbonError::UnauthorizedOracle);
        }
        Ok(())
    }
}

mod test;
