use crate::DeviceDataInternal;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk::api::management_canister::main::CanisterInstallMode::Install;
use ic_cdk::api::management_canister::main::{
    canister_status, create_canister, install_code, CanisterIdRecord, CanisterInstallMode,
    CanisterStatusResponse, CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::{call, id, notify};
use ic_types::CanisterId;
use internet_identity_interface::{
    ArchiveInit, DeviceData, DeviceDataUpdate, DeviceProtection, LogEntry,
};
use lazy_static::lazy_static;
use sha2::Digest;
use sha2::Sha256;
use ArchiveState::NotCreated;
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
#[derive(Clone, CandidType, Deserialize)]
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

pub fn write_entry(archive_canister: Principal, operation: LogEntry) {
    let encoded_entry = candid::encode_one(operation).expect("failed to encode log entry");
    // Notify only fails if the message cannot be enqueued.
    notify(archive_canister, "write_entry", encoded_entry)
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

pub fn calculate_device_diff(old: &DeviceDataInternal, new: &DeviceData) -> DeviceDataUpdate {
    let new_protection = new.protection.clone();
    let protection_diff = match old.protection {
        None => {
            if new_protection == DeviceProtection::Unprotected {
                None
            } else {
                Some(new_protection)
            }
        }
        Some(ref old_protection) => {
            if old_protection == new_protection {
                None
            } else {
                Some(new_protection)
            }
        }
    };

    DeviceDataUpdate {
        alias: if old.alias == new.alias {
            None
        } else {
            Some(Hidden)
        },
        credential_id: if old.credential_id == new.credential_id {
            None
        } else {
            new.credential_id.clone()
        },
        purpose: if old.purpose == new.purpose {
            None
        } else {
            new.purpose.clone()
        },
        key_type: if old.key_type == new.key_type {
            None
        } else {
            new.key_type.clone()
        },
        protection: protection_diff,
    }
}
