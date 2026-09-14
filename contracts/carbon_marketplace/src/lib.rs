#![no_std]

mod error;
pub use error::CarbonError;

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, IntoVal, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Listing {
    pub listing_id: u64,
    pub batch_id: u64,
    pub seller: Address,
    pub price_per_tonne: i128, // in USDC stroops (7 decimals)
    pub tonnes: u64,
    pub active: bool,
}

const LISTING: Symbol = Symbol::short("LISTING");
const NEXT_ID: Symbol = Symbol::short("NEXT_ID");
const ALL_IDS: Symbol = Symbol::short("ALL_IDS");
const CREDIT_CONTRACT: Symbol = Symbol::short("CREDITC");
const USDC_TOKEN: Symbol = Symbol::short("USDC");
const FEE_BPS: i128 = 100; // 1% protocol fee, matches original spec
const FEE_RECIPIENT: Symbol = Symbol::short("FEE_TO");
const ADMIN: Symbol = Symbol::short("ADMIN");
const SELLER_LISTINGS: Symbol = Symbol::short("SEL_LIST"); // per-seller listing id index
const BATCH_SIZE_MAX: u32 = 50;

#[contract]
pub struct CarbonMarketplace;

#[contractimpl]
impl CarbonMarketplace {
    /// One-time setup. Reverts if already initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        credit_contract: Address,
        usdc_token: Address,
        fee_recipient: Address,
    ) -> Result<(), CarbonError> {
        if env.storage().instance().has(&ADMIN) {
            return Err(CarbonError::ProjectAlreadyExists);
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&CREDIT_CONTRACT, &credit_contract);
        env.storage().instance().set(&USDC_TOKEN, &usdc_token);
        env.storage().instance().set(&FEE_RECIPIENT, &fee_recipient);
        env.storage().instance().set(&NEXT_ID, &1u64);
        env.storage()
            .instance()
            .set(&ALL_IDS, &Vec::<u64>::new(&env));
        Ok(())
    }

    /// Admin updates the protocol fee recipient (e.g. treasury rotation).
    pub fn set_fee_recipient(env: Env, admin: Address, new_recipient: Address) -> Result<(), CarbonError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(CarbonError::UnauthorizedVerifier)?;
        admin.require_auth();
        if stored_admin != admin {
            return Err(CarbonError::UnauthorizedVerifier);
        }
        env.storage().instance().set(&FEE_RECIPIENT, &new_recipient);
        Ok(())
    }

    /// List a credit batch for sale at a price per tonne (in USDC stroops).
    pub fn list_credits(
        env: Env,
        seller: Address,
        batch_id: u64,
        price_per_tonne: i128,
        tonnes: u64,
    ) -> Result<u64, CarbonError> {
        seller.require_auth();
        if price_per_tonne <= 0 || tonnes == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }

        let listing_id: u64 = env.storage().instance().get(&NEXT_ID).unwrap_or(1);
        let listing = Listing {
            listing_id,
            batch_id,
            seller: seller.clone(),
            price_per_tonne,
            tonnes,
            active: true,
        };
        env.storage()
            .persistent()
            .set(&(LISTING, listing_id), &listing);
        env.storage().instance().set(&NEXT_ID, &(listing_id + 1));

        let mut ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&ALL_IDS)
            .unwrap_or_else(|| Vec::new(&env));
        ids.push_back(listing_id);
        env.storage().instance().set(&ALL_IDS, &ids);

        let mut seller_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(SELLER_LISTINGS, seller.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        seller_ids.push_back(listing_id);
        env.storage()
            .persistent()
            .set(&(SELLER_LISTINGS, seller), &seller_ids);

        env.events()
            .publish((Symbol::short("listed"), listing_id), price_per_tonne);
        Ok(listing_id)
    }

    /// Seller removes an active listing.
    pub fn delist_credits(env: Env, seller: Address, listing_id: u64) -> Result<(), CarbonError> {
        seller.require_auth();
        let mut listing = Self::get_listing(env.clone(), listing_id)?;
        if listing.seller != seller {
            return Err(CarbonError::NotOwner);
        }
        listing.active = false;
        env.storage()
            .persistent()
            .set(&(LISTING, listing_id), &listing);
        Ok(())
    }

    /// Buyer purchases an entire listing: USDC moves seller <- buyer (minus
    /// a 1% protocol fee to the fee recipient), and the credit batch is
    /// transferred buyer <- seller via a cross-contract call to
    /// carbon_credit.transfer_credits().
    pub fn purchase_credits(env: Env, buyer: Address, listing_id: u64) -> Result<(), CarbonError> {
        buyer.require_auth();
        let mut listing = Self::get_listing(env.clone(), listing_id)?;
        if !listing.active {
            return Err(CarbonError::ListingNotFound);
        }

        let total_price = listing.price_per_tonne * (listing.tonnes as i128);
        let fee = (total_price * FEE_BPS) / 10_000;
        let seller_proceeds = total_price - fee;

        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&USDC_TOKEN)
            .ok_or(CarbonError::PriceNotSet)?;
        let fee_recipient: Address = env
            .storage()
            .instance()
            .get(&FEE_RECIPIENT)
            .ok_or(CarbonError::PriceNotSet)?;
        let usdc = token::Client::new(&env, &usdc_addr);

        usdc.transfer(&buyer, &listing.seller, &seller_proceeds);
        usdc.transfer(&buyer, &fee_recipient, &fee);

        let credit_addr: Address = env
            .storage()
            .instance()
            .get(&CREDIT_CONTRACT)
            .ok_or(CarbonError::PriceNotSet)?;
        let mut args: Vec<soroban_sdk::Val> = Vec::new(&env);
        args.push_back(listing.seller.clone().into_val(&env));
        args.push_back(listing.batch_id.into_val(&env));
        args.push_back(buyer.into_val(&env));
        env.invoke_contract::<()>(&credit_addr, &Symbol::new(&env, "transfer_credits"), args);

        listing.active = false;
        env.storage()
            .persistent()
            .set(&(LISTING, listing_id), &listing);

        env.events()
            .publish((Symbol::short("purchased"), listing_id), total_price);
        Ok(())
    }

    /// Buyer purchases multiple listings in one atomic transaction. Each line
    /// item must reference an active listing whose tonnes equal the requested
    /// amount (partial fills are not yet supported by the credit contract).
    /// The 1% protocol fee is applied per line item, matching purchase_credits().
    /// The entire batch rolls back atomically if any single line item is invalid.
    pub fn bulk_purchase(
        env: Env,
        buyer: Address,
        purchases: Vec<(u64, u64)>,
    ) -> Result<(), CarbonError> {
        buyer.require_auth();

        let num_items = purchases.len();
        if num_items == 0 {
            return Err(CarbonError::ZeroAmountNotAllowed);
        }
        if num_items > BATCH_SIZE_MAX {
            return Err(CarbonError::BatchTooLarge);
        }

        // --- duplicate listing_id check ---
        for i in 0..num_items {
            let (id_i, _) = purchases.get(i).unwrap();
            for j in (i + 1)..num_items {
                let (id_j, _) = purchases.get(j).unwrap();
                if id_i == id_j {
                    return Err(CarbonError::DuplicateListingId);
                }
            }
        }

        // --- validate every listing before moving any funds/credits ---
        for i in 0..num_items {
            let (listing_id, amount) = purchases.get(i).unwrap();
            let listing = Self::get_listing(env.clone(), listing_id)?;
            if !listing.active {
                return Err(CarbonError::ListingNotFound);
            }
            if amount == 0 || amount != listing.tonnes {
                return Err(CarbonError::InsufficientCredits);
            }
        }

        // --- execute all purchases ---
        let usdc_addr: Address = env
            .storage()
            .instance()
            .get(&USDC_TOKEN)
            .ok_or(CarbonError::PriceNotSet)?;
        let fee_recipient: Address = env
            .storage()
            .instance()
            .get(&FEE_RECIPIENT)
            .ok_or(CarbonError::PriceNotSet)?;
        let credit_addr: Address = env
            .storage()
            .instance()
            .get(&CREDIT_CONTRACT)
            .ok_or(CarbonError::PriceNotSet)?;
        let usdc = token::Client::new(&env, &usdc_addr);

        let mut total_tonnes: u64 = 0;
        let mut total_usdc: i128 = 0;
        let mut total_fees: i128 = 0;

        for i in 0..num_items {
            let (listing_id, amount) = purchases.get(i).unwrap();
            let mut listing = Self::get_listing(env.clone(), listing_id)?;

            let total_price = listing.price_per_tonne * (amount as i128);
            let fee = (total_price * FEE_BPS) / 10_000;
            let seller_proceeds = total_price - fee;

            usdc.transfer(&buyer, &listing.seller, &seller_proceeds);
            usdc.transfer(&buyer, &fee_recipient, &fee);

            let mut args: Vec<soroban_sdk::Val> = Vec::new(&env);
            args.push_back(listing.seller.clone().into_val(&env));
            args.push_back(listing.batch_id.into_val(&env));
            args.push_back(buyer.clone().into_val(&env));
            env.invoke_contract::<()>(
                &credit_addr,
                &Symbol::new(&env, "transfer_credits"),
                args,
            );

            listing.active = false;
            env.storage()
                .persistent()
                .set(&(LISTING, listing_id), &listing);

            env.events()
                .publish((Symbol::short("purchased"), listing_id), total_price);

            total_tonnes += amount;
            total_usdc += total_price;
            total_fees += fee;
        }

        env.events().publish(
            (Symbol::short("BULK_PURCH"), num_items),
            (buyer, total_tonnes, total_usdc, total_fees),
        );

        Ok(())
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Result<Listing, CarbonError> {
        env.storage()
            .persistent()
            .get(&(LISTING, listing_id))
            .ok_or(CarbonError::ListingNotFound)
    }

    /// Browse all currently active listings.
    pub fn get_active_listings(env: Env) -> Vec<Listing> {
        let ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&ALL_IDS)
            .unwrap_or_else(|| Vec::new(&env));
        let mut active = Vec::new(&env);
        for id in ids.iter() {
            if let Some(listing) = env.storage().persistent().get::<_, Listing>(&(LISTING, id)) {
                if listing.active {
                    active.push_back(listing);
                }
            }
        }
        active
    }

    /// List all listing ids (active or not) created by a given seller.
    pub fn get_listings_by_seller(env: Env, seller: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&(SELLER_LISTINGS, seller))
            .unwrap_or_else(|| Vec::new(&env))
    }
}

mod test;
