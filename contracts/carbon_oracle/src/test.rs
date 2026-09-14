#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup(env: &Env) -> CarbonOracleClient {
    let contract_id = env.register_contract(None, CarbonOracle);
    let client = CarbonOracleClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let signer = Address::generate(env);
    client.initialize(&admin, &signer);
    client
}

#[test]
fn monitoring_freshness() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    assert!(!client.is_monitoring_current(&1u64));
}

#[test]
fn price_rejects_large_deviation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CarbonOracle);
    let client = CarbonOracleClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    client.initialize(&admin, &signer);

    let methodology = String::from_str(&env, "VM0007");
    client.update_credit_price(&signer, &methodology, &2024u32, &10_000_000i128);

    // 50% jump should be rejected (max allowed move is 15%).
    let result =
        client.try_update_credit_price(&signer, &methodology, &2024u32, &15_000_000i128);
    assert!(result.is_err());

    // 10% move is within tolerance.
    client.update_credit_price(&signer, &methodology, &2024u32, &10_900_000i128);
    let price = client.get_benchmark_price(&methodology, &2024u32);
    assert_eq!(price.price_per_tonne, 10_900_000i128);
}
