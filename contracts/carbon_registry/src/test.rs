#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup(env: &Env) -> (CarbonRegistryClient, Address, Address) {
    let contract_id = env.register_contract(None, CarbonRegistry);
    let client = CarbonRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin, Address::generate(env))
}

#[test]
fn cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _developer) = setup(&env);

    let another_admin = Address::generate(&env);
    let result = client.try_initialize(&another_admin);
    assert!(result.is_err());
}

#[test]
fn register_and_verify_project() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, developer) = setup(&env);
    let verifier = Address::generate(&env);
    client.add_verifier(&admin, &verifier);
    assert!(client.is_verifier(&verifier));

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(-3_465_000i64),
        &(-62_216_000i64),
    );

    let project = client.get_project(&id);
    assert_eq!(project.status, ProjectStatus::Pending);

    client.verify_project(&verifier, &id, &85);
    let project = client.get_project(&id);
    assert_eq!(project.status, ProjectStatus::Verified);
    assert_eq!(project.methodology_score, 85);
}

#[test]
fn verify_fails_below_minimum_score() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, developer) = setup(&env);
    let verifier = Address::generate(&env);
    client.add_verifier(&admin, &verifier);

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0015"),
        &(0i64),
        &(0i64),
    );

    let result = client.try_verify_project(&verifier, &id, &50);
    assert!(result.is_err());
}

#[test]
fn unauthorized_verifier_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, developer) = setup(&env);
    let not_a_verifier = Address::generate(&env);

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(0i64),
        &(0i64),
    );

    let result = client.try_verify_project(&not_a_verifier, &id, &90);
    assert!(result.is_err());
}

#[test]
fn rejected_project_cannot_be_verified() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, developer) = setup(&env);
    let verifier = Address::generate(&env);
    client.add_verifier(&admin, &verifier);

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(0i64),
        &(0i64),
    );
    client.reject_project(&verifier, &id);

    let result = client.try_verify_project(&verifier, &id, &90);
    assert!(result.is_err());

    let result = client.try_reject_project(&verifier, &id);
    assert!(result.is_err());
}

#[test]
fn suspended_project_can_be_reverified() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, developer) = setup(&env);
    let verifier = Address::generate(&env);
    client.add_verifier(&admin, &verifier);

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(0i64),
        &(0i64),
    );
    client.verify_project(&verifier, &id, &80);
    client.suspend_project(&verifier, &id);
    assert_eq!(client.get_project(&id).status, ProjectStatus::Suspended);

    client.verify_project(&verifier, &id, &82);
    assert_eq!(client.get_project(&id).status, ProjectStatus::Verified);
}

#[test]
fn removed_verifier_loses_access() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, developer) = setup(&env);
    let verifier = Address::generate(&env);
    client.add_verifier(&admin, &verifier);
    client.remove_verifier(&admin, &verifier);
    assert!(!client.is_verifier(&verifier));

    let id = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(0i64),
        &(0i64),
    );
    let result = client.try_verify_project(&verifier, &id, &90);
    assert!(result.is_err());
}

#[test]
fn only_admin_can_manage_verifiers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _developer) = setup(&env);
    let stranger = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_add_verifier(&stranger, &target);
    assert!(result.is_err());
}

#[test]
fn admin_transfer_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _developer) = setup(&env);
    let new_admin = Address::generate(&env);

    client.transfer_admin(&admin, &new_admin);
    assert_eq!(client.get_admin(), new_admin);

    let target = Address::generate(&env);
    let result = client.try_add_verifier(&admin, &target);
    assert!(result.is_err());
}

#[test]
fn tracks_projects_by_developer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, developer) = setup(&env);

    let id1 = client.register_project(
        &developer,
        &String::from_str(&env, "VM0007"),
        &(0i64),
        &(0i64),
    );
    let id2 = client.register_project(
        &developer,
        &String::from_str(&env, "VM0015"),
        &(0i64),
        &(0i64),
    );

    let ids = client.get_projects_by_developer(&developer);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), id1);
    assert_eq!(ids.get(1).unwrap(), id2);
}
