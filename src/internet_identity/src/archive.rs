use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::main::{
    canister_status, create_canister, install_code, CanisterIdRecord, CanisterInstallMode,
    CanisterStatusResponse, CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::{call, id, trap};
use internet_identity_interface::ArchiveInit;
use lazy_static::lazy_static;
use sha2::Digest;
use sha2::Sha256;
use CanisterInstallMode::Upgrade;

lazy_static! {
    static ref ARCHIVE_HASH: [u8; 32] =
        hex::decode("3e7af31f4bacf515ebe3c5befa9e7a836f36aa7441a9cbcfdaafa8b7bfd1cbe7")
            .unwrap()
            .try_into()
            .unwrap();
}

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
    let (result,) = create_canister(CreateCanisterArgument { settings: None })
        .await
        .expect("failed to create archive canister");

    ArchiveData {
        archive_seq_number: 0,
        archive_canister: result.canister_id,
    }
}

pub async fn upgrade_archive(archive_canister: Principal, wasm_module: Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update(&wasm_module);
    let wasm_hash: [u8; 32] = hasher.finalize().into();

    if wasm_hash != ARCHIVE_HASH.clone() {
        trap("invalid wasm module")
    }

    let (archive_status,) = canister_status(CanisterIdRecord {
        canister_id: archive_canister,
    })
    .await
    .expect("failed to retrieve archive status");

    if archive_status
        .module_hash
        .map(|hash| hash == ARCHIVE_HASH.clone())
        .unwrap_or(false)
    {
        // the canister already has the given module installed --> don't do anything
        return;
    }

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
