use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterInstallMode::Install;
use ic_cdk::api::management_canister::main::{
    canister_status, create_canister, install_code, CanisterIdRecord, CanisterInstallMode,
    CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::{id, notify};
use internet_identity_interface::{ArchiveInit, LogEntry};
use lazy_static::lazy_static;
use sha2::Digest;
use sha2::Sha256;
use ArchiveState::NotCreated;
use CanisterInstallMode::Upgrade;

lazy_static! {
    static ref ARCHIVE_HASH: [u8; 32] =
        hex::decode("039041594a2d0b6dd0a3f18a7406cbc65c93303d9f9eef5635aa30434e7643a5")
            .unwrap()
            .try_into()
            .unwrap();
}

/// State of the archive canister.
#[derive(Eq, PartialEq, Clone, CandidType, Deserialize)]
pub enum ArchiveState {
    NotCreated,           // Archive has not been created.
    CreationInProgress,   // Archive is being created.
    Created(ArchiveData), // Archive exists.
}

impl Default for ArchiveState {
    fn default() -> Self {
        NotCreated
    }
}

/// Management metadata about the archive.
#[derive(Eq, PartialEq, Clone, CandidType, Deserialize)]
pub struct ArchiveData {
    // Sequence number of anchor operations. Using this sequence number missing entries / reliability
    // can be assessed without having explicit error handling on the II side.
    pub sequence_number: u64,
    // Canister id of the archive canister
    pub archive_canister: Principal,
}

pub async fn create_archive() -> CallResult<ArchiveData> {
    let (result,) = create_canister(CreateCanisterArgument { settings: None }).await?;
    Ok(ArchiveData {
        sequence_number: 0,
        archive_canister: result.canister_id,
    })
}

pub async fn install_archive(
    archive_canister: Principal,
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    verify_wasm_hash(&wasm_module)?;

    let (archive_status,) = canister_status(CanisterIdRecord {
        canister_id: archive_canister,
    })
    .await
    .map_err(|err| format!("failed to retrieve archive status: {:?}", err))?;

    let module_hash = archive_status.module_hash;
    if module_hash
        .clone()
        .map(|hash| hash == ARCHIVE_HASH.clone())
        .unwrap_or(false)
    {
        // Don't do anything further if the archive canister has the given module already installed
        return Ok(());
    }

    let mode = match module_hash {
        None => Install,
        Some(_) => Upgrade,
    };

    let settings = ArchiveInit {
        ii_canister: id(),
        max_entries_per_call: 1000,
    };
    let encoded_arg = candid::encode_one(settings)
        .map_err(|err| format!("failed to encode archive install argument: {:?}", err))?;

    install_code(InstallCodeArgument {
        mode,
        canister_id: archive_canister,
        wasm_module,
        arg: encoded_arg,
    })
    .await
    .map_err(|err| format!("failed to install archive canister: {:?}", err))
}

pub fn write_entry(archive_canister: Principal, entry: LogEntry) {
    let user_number = entry.user_number;
    let timestamp = entry.timestamp;
    let encoded_entry = candid::encode_one(entry).expect("failed to encode log entry");
    // Notify only fails if the message cannot be enqueued.
    let () = notify(
        archive_canister,
        "write_entry",
        (user_number, timestamp, encoded_entry),
    )
    .expect("failed to send log entry notification");
}

fn verify_wasm_hash(wasm_module: &Vec<u8>) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(&wasm_module);
    let wasm_hash: [u8; 32] = hasher.finalize().into();

    if wasm_hash != ARCHIVE_HASH.clone() {
        return Err("invalid wasm module".to_string());
    }
    Ok(())
}
