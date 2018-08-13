use azure::core::errors::{check_status_extract_headers_and_body, AzureError};
use azure::core::lease::LeaseId;
use azure::core::{
    BlobNameRequired, BlobNameSupport, BlockIdRequired, BlockIdSupport, BodyRequired, BodySupport, ClientRequestIdOption,
    ClientRequestIdSupport, ClientRequired, ContainerNameRequired, ContainerNameSupport, ContentMD5Option, ContentMD5Support,
    LeaseIdOption, LeaseIdSupport, No, TimeoutOption, TimeoutSupport, ToAssign, Yes,
};
use azure::storage::blob::responses::PutBlockResponse;
use azure::storage::client::Client;
use futures::future::done;
use futures::prelude::*;
use hyper::{Method, StatusCode};
use std::marker::PhantomData;
pub struct PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    client: &'a Client,
    p_container_name: PhantomData<ContainerNameSet>,
    p_blob_name: PhantomData<BlobNameSet>,
    p_body: PhantomData<BodySet>,
    p_block_id: PhantomData<BlockIdSet>,
    container_name: Option<&'a str>,
    blob_name: Option<&'a str>,
    body: Option<&'a [u8]>,
    block_id: Option<&'a str>,
    timeout: Option<u64>,
    content_md5: Option<&'a [u8]>,
    lease_id: Option<&'a LeaseId>,
    client_request_id: Option<&'a str>,
}

impl<'a> PutBlockBuilder<'a, No, No, No, No> {
    #[inline]
    pub(crate) fn new(client: &'a Client) -> PutBlockBuilder<'a, No, No, No, No> {
        PutBlockBuilder {
            client,
            p_container_name: PhantomData {},
            container_name: None,
            p_blob_name: PhantomData {},
            blob_name: None,
            p_body: PhantomData {},
            body: None,
            p_block_id: PhantomData {},
            block_id: None,
            timeout: None,
            content_md5: None,
            lease_id: None,
            client_request_id: None,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ClientRequired<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn client(&self) -> &'a Client {
        self.client
    }
}

impl<'a, BlobNameSet, BodySet, BlockIdSet> ContainerNameRequired<'a> for PutBlockBuilder<'a, Yes, BlobNameSet, BodySet, BlockIdSet>
where
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn container_name(&self) -> &'a str {
        self.container_name.unwrap()
    }
}

impl<'a, ContainerNameSet, BodySet, BlockIdSet> BlobNameRequired<'a> for PutBlockBuilder<'a, ContainerNameSet, Yes, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn blob_name(&self) -> &'a str {
        self.blob_name.unwrap()
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BlockIdSet> BodyRequired<'a> for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, Yes, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn body(&self) -> &'a [u8] {
        self.body.unwrap()
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet> BlockIdRequired<'a> for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, Yes>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
{
    #[inline]
    fn block_id(&self) -> &'a str {
        self.block_id.unwrap()
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> TimeoutOption
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn timeout(&self) -> Option<u64> {
        self.timeout
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ContentMD5Option<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn content_md5(&self) -> Option<&'a [u8]> {
        self.content_md5
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> LeaseIdOption<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn lease_id(&self) -> Option<&'a LeaseId> {
        self.lease_id
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ClientRequestIdOption<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    #[inline]
    fn client_request_id(&self) -> Option<&'a str> {
        self.client_request_id
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ContainerNameSupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, Yes, BlobNameSet, BodySet, BlockIdSet>;

    #[inline]
    fn with_container_name(self, container_name: &'a str) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: Some(container_name),
            blob_name: self.blob_name,
            body: self.body,
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> BlobNameSupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, Yes, BodySet, BlockIdSet>;

    #[inline]
    fn with_blob_name(self, blob_name: &'a str) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: Some(blob_name),
            body: self.body,
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> BodySupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, Yes, BlockIdSet>;

    #[inline]
    fn with_body(self, body: &'a [u8]) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: Some(body),
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> BlockIdSupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, Yes>;

    #[inline]
    fn with_block_id(self, block_id: &'a str) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: self.body,
            block_id: Some(block_id),
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> TimeoutSupport
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>;

    #[inline]
    fn with_timeout(self, timeout: u64) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: self.body,
            block_id: self.block_id,
            timeout: Some(timeout),
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ContentMD5Support<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>;

    #[inline]
    fn with_content_md5(self, content_md5: &'a [u8]) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: self.body,
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: Some(content_md5),
            lease_id: self.lease_id,
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> LeaseIdSupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>;

    #[inline]
    fn with_lease_id(self, lease_id: &'a LeaseId) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: self.body,
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: Some(lease_id),
            client_request_id: self.client_request_id,
        }
    }
}

impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> ClientRequestIdSupport<'a>
    for PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{
    type O = PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>;

    #[inline]
    fn with_client_request_id(self, client_request_id: &'a str) -> Self::O {
        PutBlockBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_blob_name: PhantomData {},
            p_body: PhantomData {},
            p_block_id: PhantomData {},
            container_name: self.container_name,
            blob_name: self.blob_name,
            body: self.body,
            block_id: self.block_id,
            timeout: self.timeout,
            content_md5: self.content_md5,
            lease_id: self.lease_id,
            client_request_id: Some(client_request_id),
        }
    }
}

// methods callable regardless
impl<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet> PutBlockBuilder<'a, ContainerNameSet, BlobNameSet, BodySet, BlockIdSet>
where
    ContainerNameSet: ToAssign,
    BlobNameSet: ToAssign,
    BodySet: ToAssign,
    BlockIdSet: ToAssign,
{}

impl<'a> PutBlockBuilder<'a, Yes, Yes, Yes, Yes> {
    pub fn finalize(self) -> impl Future<Item = PutBlockResponse, Error = AzureError> {
        let mut uri = format!(
            "https://{}.blob.core.windows.net/{}/{}?comp=block",
            self.client().account(),
            self.container_name(),
            self.blob_name()
        );
        if let Some(timeout) = TimeoutOption::to_uri_parameter(&self) {
            uri = format!("{}&{}", uri, timeout);
        }
        uri = format!("{}&{}", uri, BlockIdRequired::to_uri_parameter(&self));

        trace!("uri == {:?}", uri);

        let req = self.client().perform_request(
            &uri,
            Method::PUT,
            |ref mut request| {
                ContentMD5Option::add_header(&self, request);
                LeaseIdOption::add_header(&self, request);
                ClientRequestIdOption::add_header(&self, request);
            },
            Some(self.body()),
        );

        done(req)
            .from_err()
            .and_then(move |response| check_status_extract_headers_and_body(response, StatusCode::CREATED))
            .and_then(move |(headers, _body)| done(PutBlockResponse::from_headers(&headers)))
    }
}
