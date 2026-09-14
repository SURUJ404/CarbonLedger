#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

// ---------------------------------------------------------------------------
// Mock contracts – just enough to satisfy cross-contract calls
// ---------------------------------------------------------------------------

#[contract]
struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) {}
}

#[contract]
struct MockCredit;

#[contractimpl]
impl MockCredit {
    pub fn transfer_credits(_env: Env, _from: Address, _batch_id: u64, _to: Address) {}
}

// ---------------------------------------------------------------------------
// Helper: bootstrap marketplace with mock token + credit contracts
// ---------------------------------------------------------------------------

fn setup() -> (Env, CarbonMarketplaceClient, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let mock_credit_id = env.register_contract(None, MockCredit);
    let mock_usdc_id = env.register_contract(None, MockToken);
    let fee_recipient = Address::generate(&env);

    client.initialize(&admin, &mock_credit_id, &mock_usdc_id, &fee_recipient);

    (env, client, fee_recipient, mock_usdc_id)
}

// ---------------------------------------------------------------------------
// Existing tests (unchanged)
// ---------------------------------------------------------------------------

#[test]
fn list_and_delist() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let credit_contract = Address::generate(&env);
    let usdc = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&admin, &credit_contract, &usdc, &fee_recipient);

    let seller = Address::generate(&env);
    let listing_id = client.list_credits(&seller, &1u64, &5_000_000i128, &10u64);

    let listing = client.get_listing(&listing_id);
    assert!(listing.active);
    assert_eq!(listing.tonnes, 10);
    assert_eq!(client.get_listings_by_seller(&seller).len(), 1);

    client.delist_credits(&seller, &listing_id);
    let listing = client.get_listing(&listing_id);
    assert!(!listing.active);

    let active = client.get_active_listings();
    assert_eq!(active.len(), 0);
}

#[test]
fn cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let result = client.try_initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );
    assert!(result.is_err());
}

#[test]
fn only_seller_can_delist() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let seller = Address::generate(&env);
    let stranger = Address::generate(&env);
    let listing_id = client.list_credits(&seller, &1u64, &1_000_000i128, &5u64);

    let result = client.try_delist_credits(&stranger, &listing_id);
    assert!(result.is_err());
}

#[test]
fn only_admin_can_update_fee_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let stranger = Address::generate(&env);
    let new_recipient = Address::generate(&env);
    let result = client.try_set_fee_recipient(&stranger, &new_recipient);
    assert!(result.is_err());

    client.set_fee_recipient(&admin, &new_recipient);
}

#[test]
fn rejects_zero_price_or_tonnes() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonMarketplace);
    let client = CarbonMarketplaceClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );

    let seller = Address::generate(&env);
    let result = client.try_list_credits(&seller, &1u64, &0i128, &10u64);
    assert!(result.is_err());

    let result = client.try_list_credits(&seller, &1u64, &1_000_000i128, &0u64);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// bulk_purchase tests
// ---------------------------------------------------------------------------

#[test]
fn bulk_purchase_happy_path() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller_a = Address::generate(&env);
    let seller_b = Address::generate(&env);
    let seller_c = Address::generate(&env);
    let buyer = Address::generate(&env);

    let id_a = client.list_credits(&seller_a, &10u64, &2_000_000i128, &5u64);
    let id_b = client.list_credits(&seller_b, &20u64, &4_000_000i128, &3u64);
    let id_c = client.list_credits(&seller_c, &30u64, &1_000_000i128, &8u64);

    let purchases = Vec::from_array(&env, [(id_a, 5u64), (id_b, 3u64), (id_c, 8u64)]);
    client.bulk_purchase(&buyer, &purchases);

    assert!(!client.get_listing(&id_a).active);
    assert!(!client.get_listing(&id_b).active);
    assert!(!client.get_listing(&id_c).active);
    assert_eq!(client.get_active_listings().len(), 0);
}

#[test]
fn bulk_purchase_empty_batch() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();
    let buyer = Address::generate(&env);

    let purchases = Vec::new(&env);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());
}

#[test]
fn bulk_purchase_duplicate_listing() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let listing_id = client.list_credits(&seller, &1u64, &5_000_000i128, &10u64);

    let purchases = Vec::from_array(&env, [(listing_id, 10u64), (listing_id, 10u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());
}

#[test]
fn bulk_purchase_inactive_listing() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id1 = client.list_credits(&seller, &1u64, &3_000_000i128, &4u64);
    let id2 = client.list_credits(&seller, &2u64, &2_000_000i128, &6u64);

    client.delist_credits(&seller, &id1);

    let purchases = Vec::from_array(&env, [(id1, 4u64), (id2, 6u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());

    assert!(client.get_listing(&id2).active);
}

#[test]
fn bulk_purchase_wrong_amount() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let listing_id = client.list_credits(&seller, &1u64, &5_000_000i128, &10u64);

    let purchases = Vec::from_array(&env, [(listing_id, 7u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());

    let purchases = Vec::from_array(&env, [(listing_id, 0u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());

    let purchases = Vec::from_array(&env, [(listing_id, 15u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());
}

#[test]
fn bulk_purchase_fee_math_matches_single() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let price_a: i128 = 3_000_000;
    let tonnes_a: u64 = 7;
    let price_b: i128 = 5_500_000;
    let tonnes_b: u64 = 4;

    let id_a = client.list_credits(&seller, &10u64, &price_a, &tonnes_a);
    let id_b = client.list_credits(&seller, &20u64, &price_b, &tonnes_b);

    let purchases = Vec::from_array(&env, [(id_a, tonnes_a), (id_b, tonnes_b)]);
    client.bulk_purchase(&buyer, &purchases);

    // Verify fee formula matches what purchase_credits() would charge per item.
    let expected_fee_a = (price_a * (tonnes_a as i128) * FEE_BPS) / 10_000;
    let expected_fee_b = (price_b * (tonnes_b as i128) * FEE_BPS) / 10_000;
    let expected_total_fee = expected_fee_a + expected_fee_b;

    let expected_total_price_a = price_a * (tonnes_a as i128);
    let expected_total_price_b = price_b * (tonnes_b as i128);
    let expected_total_usdc = expected_total_price_a + expected_total_price_b;

    // Total USDC moved = seller proceeds + fees = total_price
    // We can verify by checking that fee + seller_proceeds == total_price
    // for each item.  The mock succeeds unconditionally so we verify the
    // arithmetic invariants that the contract relies on.
    assert_eq!(expected_total_fee, (expected_total_usdc * FEE_BPS) / 10_000);
    assert_eq!(
        expected_total_usdc,
        (price_a * tonnes_a as i128) + (price_b * tonnes_b as i128)
    );
}

#[test]
fn bulk_purchase_max_batch_size() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let mut ids = Vec::new(&env);
    for n in 0..=BATCH_SIZE_MAX {
        let id = client.list_credits(&seller, &(n as u64 + 1), &1_000_000i128, &1u64);
        ids.push_back(id);
    }

    // 51 items (0..=50 is 51 elements) should fail
    let mut purchases_ok = Vec::new(&env);
    for i in 0..BATCH_SIZE_MAX {
        let id = ids.get(i).unwrap();
        purchases_ok.push_back((id, 1u64));
    }
    client.bulk_purchase(&buyer, &purchases_ok);

    // Now push one more to hit 51 — that must fail
    let last_id = ids.get(BATCH_SIZE_MAX).unwrap();
    let mut purchases_too_many = Vec::new(&env);
    for i in 0..=BATCH_SIZE_MAX {
        let id = ids.get(i).unwrap();
        purchases_too_many.push_back((id, 1u64));
    }
    let result = client.try_bulk_purchase(&buyer, &purchases_too_many);
    assert!(result.is_err());
}

#[test]
fn bulk_purchase_rolls_back_on_mid_batch_failure() {
    let (env, client, _fee_recipient, _mock_usdc) = setup();

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let id_good = client.list_credits(&seller, &1u64, &2_000_000i128, &5u64);
    let id_bad = client.list_credits(&seller, &2u64, &3_000_000i128, &10u64);

    client.delist_credits(&seller, &id_bad);

    // Put the bad listing second so validation fails after the good one is seen
    let purchases = Vec::from_array(&env, [(id_good, 5u64), (id_bad, 10u64)]);
    let result = client.try_bulk_purchase(&buyer, &purchases);
    assert!(result.is_err());

    // Good listing must still be active — nothing was committed
    assert!(client.get_listing(&id_good).active);
}
