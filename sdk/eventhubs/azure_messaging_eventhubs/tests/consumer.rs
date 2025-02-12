// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

//cspell: words eventdata

use async_std::future::timeout;
use azure_core_test::recorded;
use azure_identity::DefaultAzureCredential;
use azure_messaging_eventhubs::consumer::{
    ConsumerClient, ConsumerClientOptions, ReceiveOptions, StartPosition,
};
use futures::{pin_mut, StreamExt};
use std::{env, error::Error, time::Duration};
use tracing::{info, trace};

mod common;

#[recorded::test(live)]
async fn test_new() -> Result<(), Box<dyn Error>> {
    common::setup();
    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;
    let _client = ConsumerClient::new(
        host,
        eventhub,
        None,
        DefaultAzureCredential::new()?,
        Some(ConsumerClientOptions {
            application_id: Some("test_new".to_string()),
            ..Default::default()
        }),
    );

    Ok(())
}

#[recorded::test(live)]
async fn test_new_with_error() -> Result<(), Box<dyn Error>> {
    common::setup();
    trace!("test_new_with_error");
    let eventhub = env::var("EVENTHUB_NAME")?;
    let consumer = ConsumerClient::new(
        "invalid_host".into(),
        eventhub,
        None,
        DefaultAzureCredential::new()?,
        Some(ConsumerClientOptions {
            application_id: Some("test_new".to_string()),
            ..Default::default()
        }),
    );
    let result = consumer.open().await;
    assert!(result.is_err());
    info!("Error: {:?}", result);

    Ok(())
}

#[recorded::test(live)]
async fn test_open() -> Result<(), Box<dyn Error>> {
    common::setup();
    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;
    let client = ConsumerClient::new(
        host,
        eventhub,
        None,
        azure_identity::DefaultAzureCredential::new()?,
        Some(ConsumerClientOptions {
            application_id: Some("test_open".to_string()),
            ..Default::default()
        }),
    );
    client.open().await?;

    Ok(())
}
#[recorded::test(live)]
async fn test_close() -> Result<(), Box<dyn Error>> {
    common::setup();
    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;
    let client = ConsumerClient::new(
        host,
        eventhub,
        None,
        azure_identity::DefaultAzureCredential::new()?,
        Some(ConsumerClientOptions {
            application_id: Some("test_open".to_string()),
            ..Default::default()
        }),
    );
    client.open().await?;
    client.close().await?;

    Ok(())
}

#[recorded::test(live)]
async fn test_get_properties() -> Result<(), Box<dyn Error>> {
    common::setup();
    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;

    let credential = DefaultAzureCredential::new()?;

    let client = ConsumerClient::new(
        host,
        eventhub.clone(),
        None,
        credential,
        Some(ConsumerClientOptions {
            application_id: Some("test_open".to_string()),
            ..Default::default()
        }),
    );
    client.open().await?;
    let properties = client.get_eventhub_properties().await?;
    info!("Properties: {:?}", properties);
    assert_eq!(properties.name, eventhub);

    Ok(())
}

#[recorded::test(live)]
async fn test_get_partition_properties() -> Result<(), Box<dyn Error>> {
    common::setup();
    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;

    let credential = DefaultAzureCredential::new()?;

    let client = ConsumerClient::new(
        host,
        eventhub,
        None,
        credential,
        Some(ConsumerClientOptions {
            application_id: Some("test_open".to_string()),
            ..Default::default()
        }),
    );
    client.open().await?;
    let properties = client.get_eventhub_properties().await?;

    for partition_id in properties.partition_ids {
        let partition_properties = client
            .get_partition_properties(partition_id.clone())
            .await?;
        info!("Partition properties: {:?}", partition_properties);
        assert_eq!(partition_properties.id, partition_id);
    }

    Ok(())
}

#[recorded::test(live)]
async fn receive_lots_of_events() -> Result<(), Box<dyn Error>> {
    common::setup();

    let host = env::var("EVENTHUBS_HOST")?;
    let eventhub = env::var("EVENTHUB_NAME")?;

    info!("Establishing credentials.");

    let credential = DefaultAzureCredential::new()?;

    info!("Creating client.");
    let client = ConsumerClient::new(
        host,
        eventhub,
        None,
        credential,
        Some(ConsumerClientOptions {
            application_id: Some("test_open".to_string()),
            ..Default::default()
        }),
    );

    info!("Opening client.");
    client.open().await?;

    info!("Creating event receive stream.");
    let event_stream = client
        .receive_events_on_partition(
            "0".to_string(),
            Some(ReceiveOptions {
                start_position: Some(StartPosition {
                    location: azure_messaging_eventhubs::consumer::StartLocation::Earliest,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
        .await;

    pin_mut!(event_stream); // Needed for iteration.

    let mut count = 0;
    const TEST_DURATION: std::time::Duration = Duration::from_secs(10);

    info!("Receiving events for {:?}.", TEST_DURATION);
    // Read events from the stream for 10 seconds.
    let result = timeout(TEST_DURATION, async {
        while let Some(event) = event_stream.next().await {
            match event {
                Ok(_event) => {
                    //                    info!("Received the following message:: {:?}", event);
                    count += 1;
                }
                Err(err) => {
                    info!("Error while receiving message: {:?}", err);
                }
            }
        }
    })
    .await;

    assert!(result.is_err());
    info!("Received {count} messages.");

    Ok(())
}
