//! Windows Credential Manager operations; one record, local persistence.

use super::{Operation, PROFILE, SERVICE, windows_record_fits};
use crate::{Failure, Result};

pub(super) fn operation(operation: Operation, target: &str) -> Result<Option<Vec<u8>>> {
    use keyring_core::{Error, api::CredentialStoreApi};
    let store =
        windows_native_keyring_store::Store::new().map_err(|_| Failure::StorageUnavailable)?;
    // LocalMachine survives process/logon restarts without roaming credentials.
    let modifiers = std::collections::HashMap::from([("target", target), ("persistence", "Local")]);
    let entry = store
        .build(SERVICE, PROFILE, Some(&modifiers))
        .map_err(|_| Failure::StorageUnavailable)?;
    match operation {
        Operation::AuthorizeAccess => entry
            .get_secret()
            .map(|_| None)
            .map_err(|_| Failure::StorageUnavailable),
        Operation::Load => match entry.get_secret() {
            Ok(bytes) => Ok(Some(bytes)),
            Err(Error::NoEntry) => Ok(None),
            Err(_) => Err(Failure::StorageUnavailable),
        },
        Operation::Save(bytes) | Operation::SaveInteractive(bytes) => {
            if !windows_record_fits(bytes.len()) {
                return Err(Failure::StorageTooLarge);
            }
            entry
                .set_secret(&bytes)
                .map(|()| None)
                .map_err(|error| match error {
                    Error::TooLong(_, _) => Failure::StorageTooLarge,
                    _ => Failure::StorageUnavailable,
                })
        }
        Operation::Clear => match entry.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(None),
            Err(_) => Err(Failure::StorageUnavailable),
        },
    }
}
