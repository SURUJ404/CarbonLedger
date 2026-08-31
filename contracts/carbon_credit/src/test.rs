#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup(env: &Env) -> (CarbonCreditClient, Address) {
    let contract_id = env.register_contract(None, CarbonCredit);
    let client = CarbonCreditClient::new(env, &contract_id);
    let minter = Address::generate(env);
    client.initialize(&minter);
    (client, minter)
}

#[test]
fn cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _minter) = setup(&env);
    let other = Address::generate(&env);
    let result = client.try_initialize(&other);
    assert!(result.is_err());
}

#[test]
fn mint_transfer_retire_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);
    let buyer = Address::generate(&env);

    let batch_id = client.mint_credits(&minter, &1u64, &developer, &2024u32, &100u64);
    let batch = client.get_credit_batch(&batch_id);
    assert_eq!(batch.serial_end - batch.serial_start + 1, 100);
    assert!(!batch.retired);
    assert_eq!(client.get_batches_by_owner(&developer).len(), 1);

    client.transfer_credits(&developer, &batch_id, &buyer);
    let batch = client.get_credit_batch(&batch_id);
    assert_eq!(batch.owner, buyer);
    assert_eq!(client.get_batches_by_owner(&developer).len(), 0);
    assert_eq!(client.get_batches_by_owner(&buyer).len(), 1);

    client.retire_credits(
        &buyer,
        &batch_id,
        &String::from_str(&env, "Acme Corp"),
        &String::from_str(&env, "2024 ESG offset"),
    );
    let batch = client.get_credit_batch(&batch_id);
    assert!(batch.retired);

    let cert = client.get_retirement_certificate(&batch_id);
    assert_eq!(cert.tonnes, 100);
}

#[test]
fn cannot_retire_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);

    let batch_id = client.mint_credits(&minter, &1u64, &developer, &2024u32, &10u64);
    client.retire_credits(
        &developer,
        &batch_id,
        &String::from_str(&env, "Beneficiary"),
        &String::from_str(&env, "Reason"),
    );

    let result = client.try_retire_credits(
        &developer,
        &batch_id,
        &String::from_str(&env, "Beneficiary"),
        &String::from_str(&env, "Reason"),
    );
    assert!(result.is_err());
}

#[test]
fn cannot_transfer_after_retirement() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);
    let buyer = Address::generate(&env);

    let batch_id = client.mint_credits(&minter, &1u64, &developer, &2024u32, &10u64);
    client.retire_credits(
        &developer,
        &batch_id,
        &String::from_str(&env, "Beneficiary"),
        &String::from_str(&env, "Reason"),
    );

    let result = client.try_transfer_credits(&developer, &batch_id, &buyer);
    assert!(result.is_err());
}

#[test]
fn cannot_transfer_someone_elses_batch() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let buyer = Address::generate(&env);

    let batch_id = client.mint_credits(&minter, &1u64, &developer, &2024u32, &10u64);
    let result = client.try_transfer_credits(&stranger, &batch_id, &buyer);
    assert!(result.is_err());
}

#[test]
fn rejects_zero_tonnes_and_bad_vintage() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);

    let result = client.try_mint_credits(&minter, &1u64, &developer, &2024u32, &0u64);
    assert!(result.is_err());

    let result = client.try_mint_credits(&minter, &1u64, &developer, &1999u32, &10u64);
    assert!(result.is_err());
}

#[test]
fn only_minter_can_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _minter) = setup(&env);
    let stranger = Address::generate(&env);
    let developer = Address::generate(&env);

    let result = client.try_mint_credits(&stranger, &1u64, &developer, &2024u32, &10u64);
    assert!(result.is_err());
}

#[test]
fn minter_transfer_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let new_minter = Address::generate(&env);
    let developer = Address::generate(&env);

    client.transfer_minter(&minter, &new_minter);

    let result = client.try_mint_credits(&minter, &1u64, &developer, &2024u32, &10u64);
    assert!(result.is_err());

    let batch_id = client.mint_credits(&new_minter, &1u64, &developer, &2024u32, &10u64);
    assert_eq!(client.get_credit_batch(&batch_id).owner, developer);
}

#[test]
fn serial_ranges_never_overlap() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, minter) = setup(&env);
    let developer = Address::generate(&env);

    let batch1 = client.mint_credits(&minter, &1u64, &developer, &2024u32, &50u64);
    let batch2 = client.mint_credits(&minter, &2u64, &developer, &2024u32, &50u64);

    let b1 = client.get_credit_batch(&batch1);
    let b2 = client.get_credit_batch(&batch2);
    assert!(b1.serial_end < b2.serial_start);
    assert!(client.verify_serial_range(&b1.serial_start, &b1.serial_end));
}
