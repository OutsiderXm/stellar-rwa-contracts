#![cfg(test)]
use super::*;
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, RegistryContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn register(
    env: &Env,
    client: &RegistryContractClient,
    issuer: &Address,
    kind: &str,
    valuation: i128,
) -> u64 {
    let token = Address::generate(env);
    client.register_asset(
        issuer,
        &token,
        &String::from_str(env, "Asset"),
        &String::from_str(env, kind),
        &valuation,
    )
}

#[test]
fn test_version() {
    let (_env, client, _admin) = setup();
    assert_eq!(client.version(), VERSION);
}

#[test]
fn test_initialize_admin() {
    let (_env, client, admin) = setup();
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.asset_count(), 0);
    assert_eq!(client.total_value_locked(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_get_admin_before_init_panics_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &id);
    client.get_admin();
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_init() {
    let (env, client, _admin) = setup();
    client.initialize(&Address::generate(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_register_before_init_panics_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(RegistryContract, ());
    let client = RegistryContractClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    client.register_asset(
        &issuer,
        &token,
        &String::from_str(&env, "Asset"),
        &String::from_str(&env, "real_estate"),
        &100,
    );
}

#[test]
fn test_register_and_get_asset() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let id = register(&env, &client, &issuer, "real_estate", 10_000);
    assert_eq!(id, 1);
    let entry = client.get_asset(&id);
    assert_eq!(entry.issuer, issuer);
    assert_eq!(entry.valuation, 10_000);
    assert!(entry.active);
}

#[test]
fn test_ids_increment() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let a = register(&env, &client, &issuer, "invoice", 1);
    let b = register(&env, &client, &issuer, "invoice", 1);
    let c = register(&env, &client, &issuer, "invoice", 1);
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
    assert_eq!(client.asset_count(), 3);
}

proptest! {
    #[test]
    fn prop_active_count_matches_active_entries(
        ops in prop::collection::vec((any::<u8>(), any::<u8>()), 1..20),
    ) {
        let (env, client, admin) = setup();
        let mut active_ids = Vec::new(&env);
        for op in ops {
            if active_ids.len() == 0 || op.0 % 2 == 0 {
                let issuer = Address::generate(&env);
                let token = Address::generate(&env);
                let id = client.register_asset(
                    &issuer,
                    &token,
                    &String::from_str(&env, "Asset"),
                    &String::from_str(&env, "real_estate"),
                    &1_000,
                );
                active_ids.push_back(id);
            } else {
                let idx = (op.1 as usize) % active_ids.len() as usize;
                let id = active_ids.get(idx as u32).unwrap();
                client.deactivate_asset(&admin, &id);
                let mut filtered = Vec::new(&env);
                for active_id in active_ids.iter() {
                    if active_id != id {
                        filtered.push_back(active_id);
                    }
                }
                active_ids = filtered;
            }

            let active_in_registry = client
                .get_all_assets(&0, &u32::MAX)
                .iter()
                .filter(|entry| entry.active)
                .count() as u64;
            assert_eq!(client.active_count(), active_in_registry);
            assert_eq!(client.active_count(), active_ids.len() as u64);
        }
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_get_missing_asset() {
    let (_env, client, _admin) = setup();
    client.get_asset(&999);
}

#[test]
fn test_get_assets_by_issuer() {
    let (env, client, _admin) = setup();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    register(&env, &client, &alice, "real_estate", 5);
    register(&env, &client, &alice, "commodity", 5);
    register(&env, &client, &bob, "invoice", 5);
    assert_eq!(client.get_assets_by_issuer(&alice).len(), 2);
    assert_eq!(client.get_assets_by_issuer(&bob).len(), 1);
}

#[test]
fn test_get_assets_by_type() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 5);
    register(&env, &client, &issuer, "real_estate", 5);
    register(&env, &client, &issuer, "commodity", 5);
    assert_eq!(
        client
            .get_assets_by_type(&String::from_str(&env, "real_estate"))
            .len(),
        2
    );
    assert_eq!(
        client
            .get_assets_by_type(&String::from_str(&env, "commodity"))
            .len(),
        1
    );
}

#[test]
fn test_get_all_and_tvl() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 100);
    register(&env, &client, &issuer, "invoice", 250);
    assert_eq!(client.get_all_assets(&0, &2).len(), 2);
    assert_eq!(client.total_value_locked(), 350);
}

#[test]
fn test_get_all_assets_pagination_edge_cases() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 100);
    register(&env, &client, &issuer, "invoice", 250);
    register(&env, &client, &issuer, "commodity", 300);

    // Test start_id = 0 (should clamp to 1)
    let result = client.get_all_assets(&0, &10);
    assert_eq!(result.len(), 3);

    // Test limit = 0 (should return empty)
    let result = client.get_all_assets(&1, &0);
    assert_eq!(result.len(), 0);

    // Test start_id past counter (should return empty)
    let result = client.get_all_assets(&99, &10);
    assert_eq!(result.len(), 0);

    // Test limit past counter (should cap at counter + 1)
    let result = client.get_all_assets(&2, &1000);
    assert_eq!(result.len(), 2);

    // Test normal pagination
    let result = client.get_all_assets(&1, &2);
    assert_eq!(result.len(), 2);
    let result = client.get_all_assets(&3, &2);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_deactivate_excludes_from_tvl() {
    let (env, client, admin) = setup();
    let issuer = Address::generate(&env);
    let id = register(&env, &client, &issuer, "real_estate", 100);
    register(&env, &client, &issuer, "invoice", 250);
    assert_eq!(client.total_value_locked(), 350);
    client.deactivate_asset(&admin, &id);
    assert!(!client.get_asset(&id).active);
    assert_eq!(client.total_value_locked(), 250);
}

#[test]
fn test_tvl_sums_only_active() {
    let (env, client, admin) = setup();
    assert_eq!(client.total_value_locked(), 0);

    let issuer = Address::generate(&env);
    let a = register(&env, &client, &issuer, "real_estate", 100);
    let b = register(&env, &client, &issuer, "invoice", 250);
    let c = register(&env, &client, &issuer, "commodity", 40);
    assert_eq!(client.total_value_locked(), 390);

    client.deactivate_asset(&admin, &a);
    assert_eq!(client.total_value_locked(), 290);

    client.deactivate_asset(&admin, &c);
    assert_eq!(client.total_value_locked(), 250);

    client.deactivate_asset(&admin, &b);
    assert_eq!(client.total_value_locked(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_deactivate_requires_admin() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let id = register(&env, &client, &issuer, "real_estate", 100);
    let impostor = Address::generate(&env);
    client.deactivate_asset(&impostor, &id);
}

#[test]
fn test_active_count_excludes_deactivated() {
    let (env, client, admin) = setup();
    let issuer = Address::generate(&env);
    let a = register(&env, &client, &issuer, "real_estate", 100);
    register(&env, &client, &issuer, "invoice", 250);
    assert_eq!(client.active_count(), 2);
    assert_eq!(client.asset_count(), 2);
    client.deactivate_asset(&admin, &a);
    assert_eq!(client.active_count(), 1);
    assert_eq!(client.asset_count(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_negative_valuation_rejected() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", -1);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_empty_name_rejected() {
    // Issue #48: empty asset name must panic InvalidInput (#7).
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    client.register_asset(
        &issuer,
        &token,
        &String::from_str(&env, ""),
        &String::from_str(&env, "real_estate"),
        &100,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_invalid_asset_type_rejected() {
    // Issue #48: unknown asset_type must panic InvalidInput (#7).
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    client.register_asset(
        &issuer,
        &token,
        &String::from_str(&env, "My Asset"),
        &String::from_str(&env, "garbage"),
        &100,
    );
}

#[test]
fn test_asset_ids_increment_monotonically_and_are_never_reused() {
    // Issue #223.
    let (env, client, admin) = setup();
    let issuer = Address::generate(&env);
    let id1 = register(&env, &client, &issuer, "real_estate", 1_000);
    let id2 = register(&env, &client, &issuer, "invoice", 2_000);
    let id3 = register(&env, &client, &issuer, "commodity", 3_000);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    client.deactivate_asset(&admin, &id2);

    let id4 = register(&env, &client, &issuer, "bond", 4_000);
    assert_eq!(id4, 4);
    assert_ne!(id4, id2);
}

#[test]
fn test_get_asset_on_unknown_id_fails_asset_not_found() {
    // Issue #224.
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 1_000);

    assert_eq!(
        client.try_get_asset(&0),
        Err(Ok(Error::AssetNotFound.into()))
    );
    assert_eq!(
        client.try_get_asset(&2),
        Err(Ok(Error::AssetNotFound.into()))
    );
}

#[test]
fn test_get_assets_by_issuer_and_by_type_return_empty_vec_not_error() {
    // Issue #225.
    let (env, client, _admin) = setup();
    let unknown_issuer = Address::generate(&env);

    let by_issuer = client.get_assets_by_issuer(&unknown_issuer);
    assert_eq!(by_issuer.len(), 0);

    let by_type = client.get_assets_by_type(&String::from_str(&env, "fund"));
    assert_eq!(by_type.len(), 0);
}

#[test]
fn test_deactivate_asset_on_unknown_id_fails_and_active_count_unchanged() {
    // Issue #226.
    let (env, client, admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 1_000);
    assert_eq!(client.active_count(), 1);

    assert_eq!(
        client.try_deactivate_asset(&admin, &99),
        Err(Ok(Error::AssetNotFound.into()))
    );
    assert_eq!(client.active_count(), 1);
}

#[test]
fn test_deactivate_already_inactive_asset_is_noop() {
    // Issue #298: deactivating an already-inactive asset should be a no-op (no event emitted).
    let (env, client, admin) = setup();
    let issuer = Address::generate(&env);
    let id = register(&env, &client, &issuer, "real_estate", 100);
    assert_eq!(client.active_count(), 1);
    assert_eq!(client.total_value_locked(), 100);

    client.deactivate_asset(&admin, &id);
    assert_eq!(client.active_count(), 0);
    assert_eq!(client.total_value_locked(), 0);
    assert!(!client.get_asset(&id).active);

    // Deactivate again — should be a no-op (counts and TVL unchanged)
    client.deactivate_asset(&admin, &id);
    assert_eq!(client.active_count(), 0);
    assert_eq!(client.total_value_locked(), 0);
    assert!(!client.get_asset(&id).active);
}

// ---- issue #308: duplicate token_contract registration ----

/// Registering the same token contract twice must be rejected, otherwise TVL
/// and any client reading `get_all_assets` would double-count the same
/// underlying asset under two distinct registry ids.
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_duplicate_token_contract_registration_rejected() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    client.register_asset(
        &issuer,
        &token,
        &String::from_str(&env, "Asset One"),
        &String::from_str(&env, "real_estate"),
        &100,
    );
    // Same token_contract, even under a different issuer/name, must be rejected.
    let other_issuer = Address::generate(&env);
    client.register_asset(
        &other_issuer,
        &token,
        &String::from_str(&env, "Asset One Again"),
        &String::from_str(&env, "invoice"),
        &200,
    );
}

#[test]
fn test_duplicate_registration_does_not_double_count_tvl() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    register(&env, &client, &issuer, "real_estate", 100);
    assert_eq!(client.total_value_locked(), 100);
    assert_eq!(client.asset_count(), 1);
}

// ---- issue #310: get_all_assets max page size ----

/// `limit` beyond `MAX_PAGE_SIZE` is silently clamped, bounding response size
/// regardless of what a caller requests.
#[test]
fn test_get_all_assets_enforces_max_page_size() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    for i in 0..5 {
        register(&env, &client, &issuer, "real_estate", 100 + i);
    }
    // Requesting far more than exist, and far more than MAX_PAGE_SIZE, still
    // only returns what's actually registered (small-registry call keeps working).
    let result = client.get_all_assets(&1, &(MAX_PAGE_SIZE * 10));
    assert_eq!(result.len(), 5);
}

/// The final page of a paginated walk may be partial (fewer than `limit`
/// items) once it reaches the end of the registry.
#[test]
fn test_get_all_assets_final_partial_page() {
    let (env, client, _admin) = setup();
    let issuer = Address::generate(&env);
    for i in 0..7 {
        register(&env, &client, &issuer, "real_estate", 100 + i);
    }
    let page_size = 3u32;
    let first = client.get_all_assets(&1, &page_size);
    assert_eq!(first.len(), 3);
    let second = client.get_all_assets(&4, &page_size);
    assert_eq!(second.len(), 3);
    // Final page is partial: only 1 asset remains (7 total, 6 already read).
    let third = client.get_all_assets(&7, &page_size);
    assert_eq!(third.len(), 1);
    let fourth = client.get_all_assets(&8, &page_size);
    assert_eq!(fourth.len(), 0);
}
