// Copyright (C) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

//! # Event Hubs SDK for Rust
//!
//!

mod common;
use azure_core::error::ErrorKind as AzureErrorKind;
use azure_messaging_eventhubs::CheckpointStore;
use std::sync::Arc;

use azure_messaging_eventhubs::{
    models::{Checkpoint, Ownership},
    InMemoryCheckpointStore,
};
use tracing::info;

#[test]
fn test_update_ownership() {
    common::setup();
    let store = InMemoryCheckpointStore::new();
    let ownership = Ownership {
        fully_qualified_namespace: "namespace".to_string(),
        event_hub_name: "event_hub".to_string(),
        consumer_group: "consumer_group".to_string(),
        partition_id: "partition_id".to_string(),
        owner_id: Some("owner_id".to_string()),
        etag: Some("etag".into()),
        ..Default::default()
    };
    let result = store.update_ownership(&ownership);
    assert!(result.is_ok());
}

#[test]
fn test_update_ownership_invalid() {
    common::setup();
    let store = InMemoryCheckpointStore::new();
    let ownership = Ownership {
        fully_qualified_namespace: "fqdn.servicebus.windows.net".to_string(),
        partition_id: "partition_id".to_string(),
        owner_id: Some("owner_id".to_string()),
        etag: Some("etag".into()),
        ..Default::default()
    };
    let result = store.update_ownership(&ownership);
    assert!(result.is_err());
    assert_eq!(*result.unwrap_err().kind(), AzureErrorKind::Other);
}

#[tokio::test]
async fn test_update_checkpoint() {
    common::setup();
    let store = InMemoryCheckpointStore::new();
    let checkpoint = Checkpoint {
        fully_qualified_namespace: "namespace".to_string(),
        event_hub_name: "event_hub".to_string(),
        consumer_group: "consumer_group".to_string(),
        partition_id: "partition_id".to_string(),
        ..Default::default()
    };
    let result = store.update_checkpoint(checkpoint).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_checkpoints() {
    common::setup();
    let store = InMemoryCheckpointStore::new();
    let checkpoint = Checkpoint {
        fully_qualified_namespace: "namespace".to_string(),
        event_hub_name: "event_hub".to_string(),
        consumer_group: "consumer_group".to_string(),
        partition_id: "partition_id".to_string(),
        ..Default::default()
    };
    info!("Adding checkpoint: {checkpoint:?}");
    store.update_checkpoint(checkpoint).await.unwrap();

    let checkpoints = store
        .list_checkpoints("namespace", "event_hub", "consumer_group")
        .await
        .unwrap();

    info!("List checkpoints: {checkpoints:?}");
    assert_eq!(checkpoints.len(), 1);
}

fn get_random_name(prefix: &str) -> String {
    format!("{}{}", prefix, azure_core::Uuid::new_v4())
}

#[tokio::test]
async fn checkpoints() -> azure_core::Result<()> {
    common::setup();
    let test_name = get_random_name("checkpoint");

    let checkpoint_store = Arc::new(InMemoryCheckpointStore::new());
    let checkpoints = checkpoint_store
        .list_checkpoints(
            "fully-qualified-namespace",
            "event-hub-name",
            "consumer-group",
        )
        .await
        .unwrap();
    assert_eq!(checkpoints.len(), 0);

    let checkpoint = Checkpoint {
        fully_qualified_namespace: "ns.servicebus.windows.net".to_string(),
        event_hub_name: "event-hub-name".to_string(),
        consumer_group: "consumer-group".to_string(),
        partition_id: test_name.clone(),
        offset: Some("offset".to_string()),
        sequence_number: Some(0),
    };

    // Even though we added a checkpoint in one namespace, it doesn't change the older namespace.
    checkpoint_store
        .update_checkpoint(checkpoint.clone())
        .await
        .unwrap();
    let checkpoints = checkpoint_store
        .list_checkpoints(
            "fully-qualified-namespace",
            "event-hub-name",
            "consumer-group",
        )
        .await
        .unwrap();
    assert_eq!(checkpoints.len(), 0);

    let checkpoints = checkpoint_store
        .list_checkpoints(
            "ns.servicebus.windows.net",
            "event-hub-name",
            "consumer-group",
        )
        .await;
    assert!(checkpoints.is_ok());
    let checkpoints = checkpoints.unwrap();
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].partition_id, test_name.as_str());
    assert_eq!(checkpoints[0].offset, Some("offset".to_string()));
    assert_eq!(checkpoints[0].sequence_number, Some(0));
    assert_eq!(checkpoints[0].event_hub_name, "event-hub-name");
    assert_eq!(checkpoints[0].consumer_group, "consumer-group");

    Ok(())
}
