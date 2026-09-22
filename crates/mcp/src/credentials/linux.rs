//! Persistent Secret Service records, with interaction confined to explicit login.

use std::collections::HashMap;

use secret_service::{EncryptionType, SecretService};
use zbus::{Connection, Proxy, zvariant::OwnedObjectPath};

use super::{MAX_RECORD, Operation, SERVICE};
use crate::{Failure, Result};

fn storage_error(error: secret_service::Error) -> Failure {
    match error {
        secret_service::Error::Locked | secret_service::Error::Prompt => {
            Failure::StorageInteractionRequired
        }
        _ => Failure::StorageUnavailable,
    }
}

pub(super) async fn operation(operation: Operation, profile: &str) -> Result<Option<Vec<u8>>> {
    let connection = Connection::session()
        .await
        .map_err(|_| Failure::StorageUnavailable)?;
    let service = SecretService::connect_with_existing(EncryptionType::Dh, connection.clone())
        .await
        .map_err(storage_error)?;
    let collection = match service.get_default_collection().await {
        Ok(collection) => collection,
        Err(secret_service::Error::NoResult) if matches!(operation, Operation::AuthorizeAccess) => {
            service
                .create_collection("TradingView CLI", "default")
                .await
                .map_err(storage_error)?
        }
        Err(secret_service::Error::NoResult) => return Err(Failure::StorageInteractionRequired),
        Err(error) => return Err(storage_error(error)),
    };
    // A session-only collection would discard credentials at logout. Never use
    // get_any_collection, which can silently choose an unrelated store.
    match service.get_collection_by_alias("session").await {
        Ok(session) if session.collection_path == collection.collection_path => {
            return Err(Failure::StorageUnavailable);
        }
        Ok(_) | Err(secret_service::Error::NoResult) => {}
        Err(error) => return Err(storage_error(error)),
    }
    if matches!(operation, Operation::AuthorizeAccess) {
        collection.unlock().await.map_err(storage_error)?;
    }
    collection.ensure_unlocked().await.map_err(storage_error)?;
    let attributes = HashMap::from([
        ("service", SERVICE),
        ("profile", profile),
        ("xdg:schema", "org.freedesktop.Secret.Generic"),
    ]);
    let mut items = collection
        .search_items(attributes.clone())
        .await
        .map_err(storage_error)?;
    // Secret Service searches are subset matches. Do not adopt a record with
    // additional ownership attributes or choose arbitrarily among duplicates.
    if items.len() > 1 {
        return Err(Failure::StorageUnavailable);
    }
    let item = items.pop();
    if let Some(item) = &item {
        let actual = item.get_attributes().await.map_err(storage_error)?;
        if actual.len() != attributes.len()
            || attributes
                .iter()
                .any(|(key, value)| actual.get(*key).map(String::as_str) != Some(*value))
        {
            return Err(Failure::StorageUnavailable);
        }
        if matches!(operation, Operation::AuthorizeAccess) {
            item.unlock().await.map_err(storage_error)?;
        }
        item.ensure_unlocked().await.map_err(storage_error)?;
    }
    let interactive = matches!(operation, Operation::SaveInteractive(_));
    match operation {
        Operation::AuthorizeAccess => Ok(None),
        Operation::Load => match item {
            None => Ok(None),
            Some(item) => {
                let bytes = item.get_secret().await.map_err(storage_error)?;
                if bytes.len() > MAX_RECORD {
                    return Err(Failure::StorageTooLarge);
                }
                Ok(Some(bytes))
            }
        },
        Operation::Save(bytes) | Operation::SaveInteractive(bytes) => {
            if bytes.len() > MAX_RECORD {
                return Err(Failure::StorageTooLarge);
            }
            let saved = match item {
                Some(item) => {
                    // SetSecret has no prompt result and updates the item in place.
                    // Never delete the old item before attempting replacement.
                    item.set_secret(&bytes, "application/json")
                        .await
                        .map_err(storage_error)?;
                    item
                }
                None if interactive => collection
                    .create_item(SERVICE, attributes, &bytes, false, "application/json")
                    .await
                    .map_err(storage_error)?,
                None => return Err(Failure::StorageInteractionRequired),
            };
            if saved.get_secret().await.map_err(storage_error)? != bytes {
                return Err(Failure::StorageUnavailable);
            }
            Ok(None)
        }
        Operation::Clear => {
            if let Some(item) = item {
                delete_without_prompt(&connection, item.item_path.as_str()).await?;
            }
            Ok(None)
        }
    }
}

async fn delete_without_prompt(connection: &Connection, path: &str) -> Result<()> {
    let item = Proxy::new(
        connection,
        "org.freedesktop.secrets",
        path,
        "org.freedesktop.Secret.Item",
    )
    .await
    .map_err(|_| Failure::StorageUnavailable)?;
    let prompt: OwnedObjectPath = item
        .call("Delete", &())
        .await
        .map_err(|_| Failure::StorageUnavailable)?;
    if prompt.as_str() != "/" {
        // Do not call Prompt: even an unlocked item may ask for confirmation.
        // Dismiss the pending request and leave the caller an explicit failure.
        let prompt = Proxy::new(
            connection,
            "org.freedesktop.secrets",
            prompt.as_str(),
            "org.freedesktop.Secret.Prompt",
        )
        .await
        .map_err(|_| Failure::StorageUnavailable)?;
        prompt
            .call::<_, _, ()>("Dismiss", &())
            .await
            .map_err(|_| Failure::StorageUnavailable)?;
        return Err(Failure::StorageInteractionRequired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    // Run only in the disposable session created by the integration script.
    fn isolated_session() {
        assert_eq!(
            std::env::var("TV_MCP_ISOLATED_SECRET_SERVICE").as_deref(),
            Ok("1")
        );
    }

    #[tokio::test]
    #[ignore = "requires isolated D-Bus and an unlocked synthetic Secret Service"]
    async fn native_records_preserve_identity_and_failed_replacements() {
        isolated_session();
        let profile = "synthetic-linux-test";
        assert_eq!(operation(Operation::Load, profile).await.unwrap(), None);
        assert_eq!(
            operation(Operation::Save(vec![1]), profile).await,
            Err(Failure::StorageInteractionRequired)
        );
        operation(Operation::SaveInteractive(vec![1, 2, 3]), profile)
            .await
            .unwrap();
        operation(Operation::Save(vec![4, 5]), profile)
            .await
            .unwrap();
        assert_eq!(
            operation(Operation::Save(vec![0; MAX_RECORD + 1]), profile).await,
            Err(Failure::StorageTooLarge)
        );
        assert_eq!(
            operation(Operation::Load, profile).await.unwrap(),
            Some(vec![4, 5])
        );

        let service = SecretService::connect(EncryptionType::Dh).await.unwrap();
        let collection = service.get_default_collection().await.unwrap();
        let attributes = HashMap::from([
            ("service", SERVICE),
            ("profile", profile),
            ("xdg:schema", "org.freedesktop.Secret.Generic"),
        ]);
        let duplicate = collection
            .create_item(
                "Synthetic duplicate",
                attributes.clone(),
                &[9],
                false,
                "application/json",
            )
            .await
            .unwrap();
        assert_eq!(
            operation(Operation::Load, profile).await,
            Err(Failure::StorageUnavailable)
        );
        assert_eq!(
            operation(Operation::Clear, profile).await,
            Err(Failure::StorageUnavailable)
        );
        duplicate.delete().await.unwrap();
        let original = collection
            .search_items(attributes)
            .await
            .unwrap()
            .pop()
            .unwrap();
        original
            .set_attributes(HashMap::from([
                ("service", SERVICE),
                ("profile", profile),
                ("xdg:schema", "org.freedesktop.Secret.Generic"),
                ("other-owner", "fixture"),
            ]))
            .await
            .unwrap();
        assert_eq!(
            operation(Operation::Clear, profile).await,
            Err(Failure::StorageUnavailable)
        );
        original
            .set_attributes(HashMap::from([
                ("service", SERVICE),
                ("profile", profile),
                ("xdg:schema", "org.freedesktop.Secret.Generic"),
            ]))
            .await
            .unwrap();
        operation(Operation::Clear, profile).await.unwrap();
        operation(Operation::Clear, profile).await.unwrap();
        assert_eq!(operation(Operation::Load, profile).await.unwrap(), None);

        // A provider may use any object path for its ephemeral session store.
        let session = service.get_collection_by_alias("session").await.unwrap();
        let connection = Connection::session().await.unwrap();
        let proxy = Proxy::new(
            &connection,
            "org.freedesktop.secrets",
            "/org/freedesktop/secrets",
            "org.freedesktop.Secret.Service",
        )
        .await
        .unwrap();
        proxy
            .call::<_, _, ()>("SetAlias", &("default", &session.collection_path))
            .await
            .unwrap();
        assert_eq!(
            operation(Operation::Load, profile).await,
            Err(Failure::StorageUnavailable)
        );
        proxy
            .call::<_, _, ()>("SetAlias", &("default", &collection.collection_path))
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires isolated synthetic Secret Service; locks its test collection"]
    async fn locked_collection_never_prompts_for_ordinary_operations() {
        isolated_session();
        let service = SecretService::connect(EncryptionType::Dh).await.unwrap();
        service
            .get_default_collection()
            .await
            .unwrap()
            .lock()
            .await
            .unwrap();
        for request in [Operation::Load, Operation::Save(vec![1]), Operation::Clear] {
            assert_eq!(
                operation(request, "synthetic-locked").await,
                Err(Failure::StorageInteractionRequired)
            );
        }
    }

    struct Prompt {
        displayed: Arc<AtomicUsize>,
        dismissed: Arc<AtomicUsize>,
    }

    #[zbus::interface(name = "org.freedesktop.Secret.Prompt")]
    impl Prompt {
        fn prompt(&self, _window: &str) {
            self.displayed.fetch_add(1, Ordering::SeqCst);
        }

        fn dismiss(&self) {
            self.dismissed.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct DeleteItem;

    #[zbus::interface(name = "org.freedesktop.Secret.Item")]
    impl DeleteItem {
        fn delete(&self) -> OwnedObjectPath {
            OwnedObjectPath::try_from("/synthetic/prompt").unwrap()
        }
    }

    #[tokio::test]
    #[ignore = "requires isolated D-Bus without a running Secret Service"]
    async fn delete_confirmation_is_dismissed_without_display() {
        isolated_session();
        let displayed = Arc::new(AtomicUsize::new(0));
        let dismissed = Arc::new(AtomicUsize::new(0));
        let _server = zbus::connection::Builder::session()
            .unwrap()
            .name("org.freedesktop.secrets")
            .unwrap()
            .serve_at("/synthetic/item", DeleteItem)
            .unwrap()
            .serve_at(
                "/synthetic/prompt",
                Prompt {
                    displayed: displayed.clone(),
                    dismissed: dismissed.clone(),
                },
            )
            .unwrap()
            .build()
            .await
            .unwrap();
        let client = Connection::session().await.unwrap();
        assert_eq!(
            delete_without_prompt(&client, "/synthetic/item").await,
            Err(Failure::StorageInteractionRequired)
        );
        assert_eq!(displayed.load(Ordering::SeqCst), 0);
        assert_eq!(dismissed.load(Ordering::SeqCst), 1);
    }
}
