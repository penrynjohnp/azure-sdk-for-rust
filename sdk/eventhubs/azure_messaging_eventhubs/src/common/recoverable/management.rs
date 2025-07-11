// Copyright (c) Microsoft Corporation. All Rights reserved
// Licensed under the MIT license.

use super::RecoverableConnection;
use crate::{common::retry_azure_operation, RetryOptions};
use azure_core::{
    error::{ErrorKind as AzureErrorKind, Result},
    http::Url,
};
use azure_core_amqp::{
    AmqpError, AmqpManagement, AmqpManagementApis, AmqpOrderedMap, AmqpSession, AmqpSessionApis,
    AmqpSimpleValue, AmqpValue,
};
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, trace, warn};

pub(crate) struct RecoverableManagementClient {
    recoverable_connection: Arc<RecoverableConnection>,
}

impl RecoverableManagementClient {
    /// Creates a new RecoverableManagementClient.
    ///
    /// # Arguments
    ///
    /// * `recoverable_connection` - The recoverable connection to use for management operations.
    pub(super) fn new(recoverable_connection: Arc<RecoverableConnection>) -> Self {
        Self {
            recoverable_connection,
        }
    }
    fn should_retry_management_response(e: &azure_core::Error) -> bool {
        match e.kind() {
            AzureErrorKind::Amqp => {
                warn!("Amqp operation failed: {:?}", e.source());
                if let Some(e) = e.source() {
                    debug!(err = ?e, "Error: {e}");

                    if let Some(amqp_error) = e.downcast_ref::<Box<AmqpError>>() {
                        RecoverableConnection::should_retry_amqp_error(amqp_error)
                    } else if let Some(amqp_error) = e.downcast_ref::<AmqpError>() {
                        RecoverableConnection::should_retry_amqp_error(amqp_error)
                    } else {
                        debug!(err=?e, "Non AMQP error: {e}");
                        false
                    }
                } else {
                    debug!("No source error found");
                    false
                }
            }
            _ => {
                debug!(err=?e, "Non AMQP error: {e}");
                false
            }
        }
    }

    pub(super) async fn create_management_client(
        connection: Arc<RecoverableConnection>,
        retry_options: &RetryOptions,
    ) -> Result<Arc<AmqpManagement>> {
        // Clients must call ensure_connection before calling ensure_management_client.

        trace!("Create management session.");
        retry_azure_operation(
            || async {
                let amqp_connection = connection.ensure_connection().await?;

                let session = AmqpSession::new();
                session.begin(amqp_connection.as_ref(), None).await?;
                trace!("Session created.");

                let management_path = connection.url.to_string() + "/$management";
                let management_path = Url::parse(&management_path)?;
                let access_token = connection
                    .authorizer
                    .authorize_path(&connection, &management_path)
                    .await?;

                trace!("Create management client.");
                let management = Arc::new(AmqpManagement::new(
                    session,
                    "eventhubs_management".to_string(),
                    access_token,
                )?);
                management.attach().await?;

                Ok(management)
            },
            retry_options,
            Some(Self::should_retry_management_response),
        )
        .await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl AmqpManagementApis for RecoverableManagementClient {
    async fn call(
        &self,
        operation_type: String,
        application_properties: AmqpOrderedMap<String, AmqpSimpleValue>,
    ) -> azure_core::Result<AmqpOrderedMap<String, AmqpValue>> {
        let result = retry_azure_operation(
            || {
                let operation_type = operation_type.clone();
                let application_properties = application_properties.clone();

                async move {
                    let result = self
                        .recoverable_connection
                        .ensure_amqp_management()
                        .await?
                        .call(operation_type, application_properties)
                        .await?;
                    Ok(result)
                }
            },
            &self.recoverable_connection.retry_options,
            Some(Self::should_retry_management_response),
        )
        .await?;
        Ok(result)
    }

    async fn attach(&self) -> azure_core::Result<()> {
        unimplemented!("AmqpManagementClient does not support attach operation");
    }

    async fn detach(self) -> azure_core::Result<()> {
        unimplemented!("AmqpManagementClient does not support detach operation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{ErrorKind, EventHubsError};
    use azure_core_amqp::error::AmqpErrorCondition;

    #[test]
    fn should_retry_management_response() {
        crate::consumer::tests::setup();

        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::TooManyRequests,
                Some("Too many requests!".into()),
            )
            .into();

            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::SwitchingProtocols,
                Some("Switcheroo".into()),
            )
            .into();
            assert!(!RecoverableManagementClient::should_retry_management_response(&error));
        }
        // Verify that an explicitly boxed error is handled correctly
        {
            let error = azure_core::Error::new(
                AzureErrorKind::Amqp,
                Box::new(AmqpError::new_management_error(
                    azure_core::http::StatusCode::TooManyRequests,
                    Some("Too many requests!".into()),
                )),
            );
            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }

        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::BadGateway,
                Some("Bad Gateway".into()),
            )
            .into();
            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::RequestTimeout,
                Some("Request Timeout".into()),
            )
            .into();
            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::RequestTimeout,
                Some("Request Timeout".into()),
            )
            .into();
            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error = AmqpError::new_management_error(
                azure_core::http::StatusCode::InternalServerError,
                Some("Internal Server Error".into()),
            )
            .into();
            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error =
                EventHubsError::from(ErrorKind::InvalidManagementResponse).into();
            assert!(!RecoverableManagementClient::should_retry_management_response(&error));
        }

        {
            let error: azure_core::Error = AmqpError::new_described_error(
                AmqpErrorCondition::ResourceLimitExceeded,
                Some("Resource Limit Exceeded".into()),
                Default::default(),
            )
            .into();

            assert!(RecoverableManagementClient::should_retry_management_response(&error));
        }
        {
            let error: azure_core::Error = AmqpError::new_described_error(
                AmqpErrorCondition::IllegalState,
                Some("Illegal State".into()),
                Default::default(),
            )
            .into();

            assert!(!RecoverableManagementClient::should_retry_management_response(&error));
        }
    }
}
