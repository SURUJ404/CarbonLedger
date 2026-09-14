#![no_std]

mod error;
pub use error::CarbonError;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

/// Lifecycle status of a registered carbon project.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectStatus {
    Pending,
    Verified,
    Suspended,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Project {
    pub id: u64,
    pub developer: Address,
    pub methodology: String,
    // Fixed-point coordinates (degrees * 1_000_000) - avoids floats on-chain.
    pub lat_micro: i64,
    pub lng_micro: i64,
    pub methodology_score: u32, // 0-100, must be >= 70 to verify
    pub status: ProjectStatus,
    pub last_monitoring_ledger: u64,
}

const PROJECT: Symbol = Symbol::short("PROJECT");
const NEXT_ID: Symbol = Symbol::short("NEXT_ID");
const ADMIN: Symbol = Symbol::short("ADMIN");
const VERIFIER: Symbol = Symbol::short("VERIFIER");
const DEV_PROJECTS: Symbol = Symbol::short("DEV_PROJ"); // per-developer project id index
const MIN_METHODOLOGY_SCORE: u32 = 70;

#[contract]
pub struct CarbonRegistry;

#[contractimpl]
impl CarbonRegistry {
    /// One-time setup. Sets the contract admin who can add/remove verifiers.
    /// Reverts if the contract has already been initialized, so this can
    /// never silently reassign the admin after deployment.
    pub fn initialize(env: Env, admin: Address) -> Result<(), CarbonError> {
        if env.storage().instance().has(&ADMIN) {
            return Err(CarbonError::ProjectAlreadyExists);
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&NEXT_ID, &1u64);
        env.events().publish((Symbol::short("init"),), admin);
        Ok(())
    }

    /// Admin grants verifier privileges to an address.
    pub fn add_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), CarbonError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&(VERIFIER, verifier.clone()), &true);
        env.events()
            .publish((Symbol::short("ver_add"),), verifier);
        Ok(())
    }

    /// Admin revokes verifier privileges from an address.
    pub fn remove_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), CarbonError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&(VERIFIER, verifier.clone()), &false);
        env.events()
            .publish((Symbol::short("ver_rm"),), verifier);
        Ok(())
    }

    /// Transfer contract admin rights to a new address. Two-step transfer
    /// is intentionally not implemented here to keep the contract small;
    /// callers should verify `new_admin` carefully before calling.
    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), CarbonError> {
        Self::require_admin(&env, &admin)?;
        env.storage().instance().set(&ADMIN, &new_admin);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, CarbonError> {
        env.storage()
            .instance()
            .get(&ADMIN)
            .ok_or(CarbonError::UnauthorizedVerifier)
    }

    pub fn is_verifier(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get(&(VERIFIER, address))
            .unwrap_or(false)
    }

    /// Project developer submits a new project for verification.
    pub fn register_project(
        env: Env,
        developer: Address,
        methodology: String,
        lat_micro: i64,
        lng_micro: i64,
    ) -> Result<u64, CarbonError> {
        developer.require_auth();

        let id: u64 = env.storage().instance().get(&NEXT_ID).unwrap_or(1);

        let project = Project {
            id,
            developer: developer.clone(),
            methodology,
            lat_micro,
            lng_micro,
            methodology_score: 0,
            status: ProjectStatus::Pending,
            last_monitoring_ledger: env.ledger().sequence() as u64,
        };

        env.storage().persistent().set(&(PROJECT, id), &project);
        env.storage().instance().set(&NEXT_ID, &(id + 1));

        let mut dev_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(DEV_PROJECTS, developer.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        dev_ids.push_back(id);
        env.storage()
            .persistent()
            .set(&(DEV_PROJECTS, developer.clone()), &dev_ids);

        env.events()
            .publish((Symbol::short("registered"), id), developer);

        Ok(id)
    }

    /// Accredited verifier approves a project. Requires methodology_score >= 70
    /// and the project to currently be Pending (idempotency guard - a
    /// project can't be re-verified or verified after rejection).
    pub fn verify_project(
        env: Env,
        verifier: Address,
        project_id: u64,
        methodology_score: u32,
    ) -> Result<(), CarbonError> {
        verifier.require_auth();
        Self::require_verifier(&env, &verifier)?;

        if methodology_score < MIN_METHODOLOGY_SCORE {
            return Err(CarbonError::ProjectNotVerified);
        }

        let mut project = Self::get_project(env.clone(), project_id)?;
        if project.status != ProjectStatus::Pending && project.status != ProjectStatus::Suspended {
            return Err(CarbonError::ProjectSuspended);
        }
        project.status = ProjectStatus::Verified;
        project.methodology_score = methodology_score;
        env.storage().persistent().set(&(PROJECT, project_id), &project);

        env.events()
            .publish((Symbol::short("verified"), project_id), methodology_score);
        Ok(())
    }

    /// Verifier permanently rejects a fraudulent or ineligible project.
    /// Rejection is terminal - a rejected project can never be verified.
    pub fn reject_project(env: Env, verifier: Address, project_id: u64) -> Result<(), CarbonError> {
        verifier.require_auth();
        Self::require_verifier(&env, &verifier)?;

        let mut project = Self::get_project(env.clone(), project_id)?;
        if project.status == ProjectStatus::Rejected {
            return Err(CarbonError::ProjectSuspended);
        }
        project.status = ProjectStatus::Rejected;
        env.storage().persistent().set(&(PROJECT, project_id), &project);
        env.events().publish((Symbol::short("rejected"), project_id), ());
        Ok(())
    }

    /// Verifier halts new issuance from a project under investigation.
    pub fn suspend_project(env: Env, verifier: Address, project_id: u64) -> Result<(), CarbonError> {
        verifier.require_auth();
        Self::require_verifier(&env, &verifier)?;

        let mut project = Self::get_project(env.clone(), project_id)?;
        if project.status == ProjectStatus::Rejected {
            return Err(CarbonError::ProjectSuspended);
        }
        project.status = ProjectStatus::Suspended;
        env.storage().persistent().set(&(PROJECT, project_id), &project);
        env.events().publish((Symbol::short("suspended"), project_id), ());
        Ok(())
    }

    /// Oracle contract pushes fresh monitoring data timestamp on-chain.
    pub fn update_monitoring(env: Env, oracle: Address, project_id: u64) -> Result<(), CarbonError> {
        oracle.require_auth();
        let mut project = Self::get_project(env.clone(), project_id)?;
        project.last_monitoring_ledger = env.ledger().sequence() as u64;
        env.storage().persistent().set(&(PROJECT, project_id), &project);
        Ok(())
    }

    /// Query full project details.
    pub fn get_project(env: Env, project_id: u64) -> Result<Project, CarbonError> {
        env.storage()
            .persistent()
            .get(&(PROJECT, project_id))
            .ok_or(CarbonError::ProjectNotFound)
    }

    /// List all project ids submitted by a given developer, in submission order.
    pub fn get_projects_by_developer(env: Env, developer: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&(DEV_PROJECTS, developer))
            .unwrap_or_else(|| Vec::new(&env))
    }

    fn require_verifier(env: &Env, verifier: &Address) -> Result<(), CarbonError> {
        let is_verifier: bool = env
            .storage()
            .persistent()
            .get(&(VERIFIER, verifier.clone()))
            .unwrap_or(false);
        if !is_verifier {
            return Err(CarbonError::UnauthorizedVerifier);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), CarbonError> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(CarbonError::UnauthorizedVerifier)?;
        if &admin != caller {
            return Err(CarbonError::UnauthorizedVerifier);
        }
        Ok(())
    }
}

mod test;
