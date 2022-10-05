use candid::Principal;
use canister_tests::api::archive as archive_api;
use canister_tests::api::internet_identity as ii_api;
use canister_tests::framework::{device_data_1, device_data_2, principal_1, CallError};
use canister_tests::{flows, framework};
use ic_state_machine_tests::{PrincipalId, StateMachine};
use ic_types::CanisterId;
use internet_identity_interface::{
    DeployArchiveResult, DeviceDataUpdate, DeviceDataWithoutAlias, DeviceProtection, Entry,
    KeyType, OperationType, Purpose,
};
use serde_bytes::ByteBuf;
use std::time::SystemTime;

const TRILLION: u128 = 1_000_000_000_000;

#[test]
fn should_deploy_archive() -> Result<(), CallError> {
    let env = StateMachine::new();
    let canister_id = framework::install_ii_canister(&env, framework::II_WASM.clone());
    // the env requires cycles to spawn canisters
    env.add_cycles(canister_id, 2 * TRILLION);

    let result = ii_api::deploy_archive(
        &env,
        canister_id,
        ByteBuf::from(framework::ARCHIVE_WASM.clone()),
    )?;

    assert!(matches!(result, DeployArchiveResult::Success));
    Ok(())
}

#[test]
fn should_record_anchor_operations() -> Result<(), CallError> {
    let env = StateMachine::new();
    let ii_canister = framework::install_ii_canister(&env, framework::II_WASM.clone());
    // the env requires cycles to spawn canisters
    env.add_cycles(ii_canister, 2 * TRILLION);

    let result = ii_api::deploy_archive(
        &env,
        ii_canister,
        ByteBuf::from(framework::ARCHIVE_WASM.clone()),
    )?;
    assert!(matches!(result, DeployArchiveResult::Success));

    let stats = ii_api::stats(&env, ii_canister)?;
    let archive_canister = CanisterId::new(PrincipalId(stats.archive.unwrap())).unwrap();
    assert!(env.canister_exists(archive_canister));

    let anchor = flows::register_anchor(&env, ii_canister);

    let mut device = device_data_2();
    ii_api::add(&env, ii_canister, principal_1(), anchor, device.clone())?;

    device.purpose = Purpose::Recovery;
    let pubkey = device.pubkey.clone();
    ii_api::update(
        &env,
        ii_canister,
        principal_1(),
        anchor,
        pubkey.clone(),
        device,
    )?;

    ii_api::remove(&env, ii_canister, principal_1(), anchor, pubkey.clone())?;

    let entries = archive_api::get_entries(&env, archive_canister, None, None)?;

    assert_eq!(entries.entries.len(), 4);

    let timestamp = env
        .time()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let register_entry = Entry {
        anchor,
        operation: OperationType::RegisterAnchor {
            device: DeviceDataWithoutAlias {
                pubkey: device_data_1().pubkey,
                credential_id: None,
                purpose: Purpose::Authentication,
                key_type: KeyType::Unknown,
                protection: DeviceProtection::Unprotected,
            },
        },
        timestamp,
        caller: Principal::from(principal_1()),
        sequence_number: 0,
    };
    assert_eq!(
        entries.entries.get(0).unwrap().as_ref().unwrap(),
        &register_entry
    );

    let add_entry = Entry {
        anchor,
        operation: OperationType::AddDevice {
            device: DeviceDataWithoutAlias::from(device_data_2()),
        },
        timestamp,
        caller: Principal::from(principal_1()),
        sequence_number: 1,
    };
    assert_eq!(
        entries.entries.get(1).unwrap().as_ref().unwrap(),
        &add_entry
    );

    let update_entry = Entry {
        anchor,
        operation: OperationType::UpdateDevice {
            device: pubkey.clone(),
            new_values: DeviceDataUpdate {
                alias: None,
                credential_id: None,
                purpose: Some(Purpose::Recovery),
                key_type: None,
                protection: None,
            },
        },
        timestamp,
        caller: Principal::from(principal_1()),
        sequence_number: 2,
    };
    assert_eq!(
        entries.entries.get(2).unwrap().as_ref().unwrap(),
        &update_entry
    );

    let delete_entry = Entry {
        anchor,
        operation: OperationType::RemoveDevice {
            device: pubkey.clone(),
        },
        timestamp,
        caller: Principal::from(principal_1()),
        sequence_number: 3,
    };
    assert_eq!(
        entries.entries.get(3).unwrap().as_ref().unwrap(),
        &delete_entry
    );
    Ok(())
}
