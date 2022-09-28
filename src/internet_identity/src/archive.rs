use candid::{CandidType, Deserialize, Principal};
use canister_tests::framework::CallError;
use ic_cdk::api::call::CallResult;
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

/// State of the archive canister.
#[derive(Clone, CandidType, Deserialize)]
pub enum ArchiveState {
    None,                 // Archive has not been created.
    CreationInProgress,   // Archive is being created.
    Created(ArchiveData), // Archive exists.
}

/// Management metadata about the archive.
#[derive(Clone, CandidType, Deserialize)]
pub struct ArchiveData {
    // Sequence number of anchor operations. Using this sequence number missing entries / reliability
    // can be assessed without having explicit error handling on the II side.
    pub archive_seq_number: u64,
    // Canister id of the archive canister
    pub archive_canister: Principal,
}

impl Default for ArchiveData {
    fn default() -> Self {
        Self
    }
}

pub async fn create_archive() -> CallResult<ArchiveData> {
    let (result,) = create_canister(CreateCanisterArgument { settings: None }).await?;

    Ok(ArchiveData {
        archive_seq_number: 0,
        archive_canister: result.canister_id,
    })
}

pub async fn upgrade_archive(
    archive_canister: Principal,
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    verify_wasm_hash(&wasm_module);

    let (archive_status,) = canister_status(CanisterIdRecord {
        canister_id: archive_canister,
    })
    .await
    .map_err(|err| format!("failed to retrieve archive status: {:?}", err))?;

    if archive_status
        .module_hash
        .map(|hash| hash == ARCHIVE_HASH.clone())
        .unwrap_or(false)
    {
        // Don't do anything if the archive canister already has the given module installed
        return Ok(());
    }

    let settings = ArchiveInit {
        ii_canister: id(),
        max_entries_per_call: 1000,
    };
    let encoded_arg = candid::encode_one(settings)
        .map_err(|err| format!("failed to encode archive install argument: {:?}", err))?;

    install_code(InstallCodeArgument {
        mode: Upgrade,
        canister_id: archive_canister,
        wasm_module,
        arg: encoded_arg,
    })
    .await
    .map_err(|err| format!("failed to upgrade archive canister: {:?}", err))
}

fn verify_wasm_hash(wasm_module: &Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update(&wasm_module);
    let wasm_hash: [u8; 32] = hasher.finalize().into();

    if wasm_hash != ARCHIVE_HASH.clone() {
        trap("invalid wasm module")
    }
}
