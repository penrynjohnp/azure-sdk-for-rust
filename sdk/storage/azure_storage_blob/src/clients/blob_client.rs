// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use crate::{
    models::BlobProperties, pipeline::StorageHeadersPolicy, BlobBlobClientGetPropertiesOptions,
    BlobClientOptions, GeneratedBlobClient,
};
use azure_core::{credentials::TokenCredential, BearerTokenCredentialPolicy, Policy, Result, Url};
use std::sync::Arc;

pub struct BlobClient {
    endpoint: Url,
    container_name: String,
    blob_name: String,
    client: GeneratedBlobClient,
}

impl BlobClient {
    pub fn new(
        endpoint: &str,
        container_name: String,
        blob_name: String,
        credential: Arc<dyn TokenCredential>,
        options: Option<BlobClientOptions>,
    ) -> Result<Self> {
        let mut options = options.unwrap_or_default();

        let storage_headers_policy = Arc::new(StorageHeadersPolicy);
        options
            .client_options
            .per_call_policies
            .push(storage_headers_policy);

        let oauth_token_policy = BearerTokenCredentialPolicy::new(
            credential.clone(),
            ["https://storage.azure.com/.default"],
        );
        options
            .client_options
            .per_try_policies
            .push(Arc::new(oauth_token_policy) as Arc<dyn Policy>);

        let client = GeneratedBlobClient::new(endpoint, credential, Some(options))?;
        Ok(Self {
            endpoint: endpoint.parse()?,
            container_name,
            blob_name,
            client,
        })
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub async fn get_blob_properties(
        &self,
        options: Option<BlobBlobClientGetPropertiesOptions<'_>>,
    ) -> Result<BlobProperties> {
        let response = self
            .client
            .get_blob_blob_client(self.container_name.clone(), self.blob_name.clone())
            .get_properties(options)
            .await?;

        let blob_properties: BlobProperties = response.headers().get()?;
        Ok(blob_properties)
    }
}
