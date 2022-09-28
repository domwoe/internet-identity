use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterIdRecord, CanisterInstallMode, CreateCanisterArgument,
    InstallCodeArgument,
};
use ic_cdk::{call, id, trap};
use internet_identity_interface::ArchiveInit;
use CanisterInstallMode::Upgrade;

/// Management metadata about the archive.
#[derive(Clone, CandidType, Deserialize)]
pub struct ArchiveData {
    // Sequence number of anchor operations. Using this sequence number missing entries / reliability
    // can be assessed without having explicit error handling on the II side.
    archive_seq_number: u64,
    // Canister id of the archive canister
    archive_canister: Principal,
}

pub async fn spawn_new_archive() -> ArchiveData {
    let result = match create_canister(CreateCanisterArgument { settings: None }).await {
        Ok((res,)) => res,
        Err((_, err)) => trap(&format!("failed to create archive canister: {}", err)),
    };

    ArchiveData {
        archive_seq_number: 0,
        archive_canister: result.canister_id,
    }
}

pub async fn upgrade_archive(archive_canister: Principal, wasm_module: Vec<u8>) {
    let settings = ArchiveInit {
        ii_canister: id(),
        max_entries_per_call: 1000,
    };
    let encoded_arg =
        candid::encode_one(settings).expect("failed to encode archive install argument");

    install_code(InstallCodeArgument {
        mode: Upgrade,
        canister_id: archive_canister,
        wasm_module,
        arg: encoded_arg,
    })
    .await
    .expect("failed to upgrade archive canister");
}
