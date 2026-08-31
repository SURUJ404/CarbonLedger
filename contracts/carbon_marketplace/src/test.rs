#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

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
