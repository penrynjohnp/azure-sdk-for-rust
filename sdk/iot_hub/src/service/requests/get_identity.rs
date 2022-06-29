use crate::service::{ServiceClient, API_VERSION};
use azure_core::error::Error;
use azure_core::Method;
use std::convert::{TryFrom, TryInto};

/// Execute the request to get the identity of a device or module.
pub(crate) async fn get_identity<T>(
    service_client: &ServiceClient,
    device_id: String,
    module_id: Option<String>,
) -> azure_core::Result<T>
where
    T: TryFrom<crate::service::CollectedResponse, Error = Error>,
{
    let uri = match module_id {
        Some(module_id) => format!(
            "https://{}.azure-devices.net/devices/{}/modules/{}?api-version={}",
            service_client.iot_hub_name, device_id, module_id, API_VERSION
        ),
        None => format!(
            "https://{}.azure-devices.net/devices/{}?api-version={}",
            service_client.iot_hub_name, device_id, API_VERSION
        ),
    };

    let mut request = service_client.finalize_request(&uri, Method::Get)?;
    request.set_body(azure_core::EMPTY_BODY);

    service_client
        .http_client()
        .execute_request_check_status(&request)
        .await?
        .try_into()
}
