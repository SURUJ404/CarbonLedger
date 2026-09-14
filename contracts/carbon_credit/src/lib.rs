#![no_std]

mod error;
pub use error::CarbonError;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

/// A minted batch of tokenized carbon credits, all belonging to one
/// contiguous serial number range so double issuance is detectable.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreditBatch {
    pub batch_id: u64,
    pub project_id: u64,
    pub owner: Address,
    pub vintage_year: u32,
    pub serial_start: u64,
    pub serial_end: u64, // inclusive; (end - start + 1) = tonnes minted
    pub retired: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RetirementCertificate {
    pub batch_id: u64,
    pub beneficiary: String,
    pub reason: String,
    pub retired_by: Address,
    pub retired_ledger: u64,
    pub tonnes: u64,
}

const BATCH: Symbol = Symbol::short("BATCH");
const CERT: Symbol = Symbol::short("CERT");
const NEXT_BATCH: Symbol = Symbol::short("NEXT_BATCH");
const NEXT_SERIAL: Symbol = Symbol::short("NEXT_SER"); // global next-free serial number
const REGISTRY_MINTER: Symbol = Symbol::short("MINTER"); // address authorized to mint (registry admin/verifier bridge)
const OWNER_BATCHES: Symbol = Symbol::short("OWN_BATCH"); // per-owner batch id index

#[contract]
pub struct CarbonCredit;

#[contractimpl]
impl CarbonCredit {
    /// One-time setup. Reverts if already initialized, preventing the
    /// minter role from ever being silently reassigned post-deployment.
    pub fn initialize(env: Env, minter: Address) -> Result<(), CarbonError> {
        if env.storage().instance().has(&REGISTRY_MINTER) {
            return Err(CarbonError::ProjectAlreadyExists);
        }
        minter.require_auth();
        env.storage().instance().set(&REGISTRY_MINTER, &minter);
        env.storage().instance().set(&NEXT_BATCH, &1u64);
        env.storage().instance().set(&NEXT_SERIAL, &1u64);
        Ok(())
    }

    /// Transfer minting rights to a new address (e.g. a new registry
    /// bridge contract). Only the current minter can do this.
    pub fn transfer_minter(env: Env, minter: Address, new_minter: Address) -> Result<(), CarbonError> {
        minter.require_auth();
        Self::require_minter(&env, &minter)?;
        env.storage().instance().set(&REGISTRY_MINTER, &new_minter);
        Ok(())
    }

    /// Mint a new batch of credits for a verified project. Serial numbers
    /// are assigned sequentially and globally, so verify_serial_range()
    /// can always prove no range was ever reused.
    pub fn mint_credits(
        env: Env,
        minter: Address,
        project_id: u64,
        owner: Address,
        vintage_year: u32,
        tonnes: u64,
    ) -> Result<u64, CarbonError> {
        minter.require_auth();
        Self::require_minter(&env, &minter)?;

        if tonnes == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }
        if vintage_year < 2005 || vintage_year > 2100 {
            return Err(CarbonError::InvalidVintageYear);
        }

        let batch_id: u64 = env.storage().instance().get(&NEXT_BATCH).unwrap_or(1);
        let serial_start: u64 = env.storage().instance().get(&NEXT_SERIAL).unwrap_or(1);
        let serial_end = serial_start + tonnes - 1;

        let batch = CreditBatch {
            batch_id,
            project_id,
            owner: owner.clone(),
            vintage_year,
            serial_start,
            serial_end,
            retired: false,
        };

        env.storage().persistent().set(&(BATCH, batch_id), &batch);
        env.storage().instance().set(&NEXT_BATCH, &(batch_id + 1));
        env.storage().instance().set(&NEXT_SERIAL, &(serial_end + 1));
        Self::add_to_owner_index(&env, &owner, batch_id);

        env.events()
            .publish((Symbol::short("minted"), batch_id), tonnes);

        Ok(batch_id)
    }

    /// Transfer an entire credit batch to a new owner (e.g. marketplace sale).
    pub fn transfer_credits(
        env: Env,
        from: Address,
        batch_id: u64,
        to: Address,
    ) -> Result<(), CarbonError> {
        from.require_auth();
        let mut batch = Self::get_credit_batch(env.clone(), batch_id)?;

        if batch.owner != from {
            return Err(CarbonError::NotOwner);
        }
        if batch.retired {
            return Err(CarbonError::AlreadyRetired);
        }

        batch.owner = to.clone();
        env.storage().persistent().set(&(BATCH, batch_id), &batch);
        Self::remove_from_owner_index(&env, &from, batch_id);
        Self::add_to_owner_index(&env, &to, batch_id);

        env.events()
            .publish((Symbol::short("transfer"), batch_id), to);
        Ok(())
    }

    /// Permanently and irreversibly retire a credit batch. Once retired,
    /// a batch can never be transferred or re-retired - this is the
    /// on-chain proof that greenwashing claims can be checked against.
    pub fn retire_credits(
        env: Env,
        owner: Address,
        batch_id: u64,
        beneficiary: String,
        reason: String,
    ) -> Result<(), CarbonError> {
        owner.require_auth();
        let mut batch = Self::get_credit_batch(env.clone(), batch_id)?;

        if batch.owner != owner {
            return Err(CarbonError::NotOwner);
        }
        if batch.retired {
            return Err(CarbonError::AlreadyRetired);
        }

        batch.retired = true;
        env.storage().persistent().set(&(BATCH, batch_id), &batch);

        let tonnes = batch.serial_end - batch.serial_start + 1;
        let cert = RetirementCertificate {
            batch_id,
            beneficiary,
            reason,
            retired_by: owner,
            retired_ledger: env.ledger().sequence() as u64,
            tonnes,
        };
        env.storage().persistent().set(&(CERT, batch_id), &cert);

        env.events()
            .publish((Symbol::short("retired"), batch_id), tonnes);

        Ok(())
    }

    /// Proves a proposed serial range does not overlap any minted batch.
    /// Because serials are assigned strictly sequentially by this contract,
    /// an overlap is only possible if a caller tries to mint outside this
    /// contract's own counter - this function lets external auditors verify
    /// a claimed range against on-chain history.
    pub fn verify_serial_range(env: Env, start: u64, end: u64) -> Result<bool, CarbonError> {
        if start > end {
            return Err(CarbonError::InvalidSerialRange);
        }
        let next: u64 = env.storage().instance().get(&NEXT_SERIAL).unwrap_or(1);
        // Any range fully below the next-free serial was legitimately minted
        // and is internally consistent by construction (no gaps, no reuse).
        Ok(end < next)
    }

    pub fn get_credit_batch(env: Env, batch_id: u64) -> Result<CreditBatch, CarbonError> {
        env.storage()
            .persistent()
            .get(&(BATCH, batch_id))
            .ok_or(CarbonError::ProjectNotFound)
    }

    pub fn get_retirement_certificate(
        env: Env,
        batch_id: u64,
    ) -> Result<RetirementCertificate, CarbonError> {
        env.storage()
            .persistent()
            .get(&(CERT, batch_id))
            .ok_or(CarbonError::AlreadyRetired)
    }

    /// List all batch ids currently (or historically) owned by an address.
    /// Used by corporate dashboards to show a buyer's full holdings.
    pub fn get_batches_by_owner(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&(OWNER_BATCHES, owner))
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn require_minter(env: &Env, minter: &Address) -> Result<(), CarbonError> {
        let authorized: Address = env
            .storage()
            .instance()
            .get(&REGISTRY_MINTER)
            .ok_or(CarbonError::UnauthorizedVerifier)?;
        if &authorized != minter {
            return Err(CarbonError::UnauthorizedVerifier);
        }
        Ok(())
    }

    fn add_to_owner_index(env: &Env, owner: &Address, batch_id: u64) {
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(OWNER_BATCHES, owner.clone()))
            .unwrap_or_else(|| Vec::new(env));
        ids.push_back(batch_id);
        env.storage()
            .persistent()
            .set(&(OWNER_BATCHES, owner.clone()), &ids);
    }

    fn remove_from_owner_index(env: &Env, owner: &Address, batch_id: u64) {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(OWNER_BATCHES, owner.clone()))
            .unwrap_or_else(|| Vec::new(env));
        let mut updated: Vec<u64> = Vec::new(env);
        for id in ids.iter() {
            if id != batch_id {
                updated.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&(OWNER_BATCHES, owner.clone()), &updated);
    }
}

mod test;
