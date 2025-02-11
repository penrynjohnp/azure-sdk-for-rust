// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#[cfg(doc)]
use crate::{models::KeyBundle, KeyClient};
use azure_core::{error::ErrorKind, Result, Url};

/// Information about the resource.
///
/// Call [`ResourceExt::resource_id()`] on supported models e.g., [`KeyBundle`] to get this information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceId {
    /// The source URL of the resource.
    pub source_id: String,

    /// The vault URL containing the resource.
    pub vault_url: String,

    /// The name of the resource.
    pub name: String,

    /// The optional version of the resource.
    pub version: Option<String>,
}

/// Extension methods to get a [`ResourceId`] from models in this crate.
pub trait ResourceExt {
    /// Gets the [`ResourceId`] from this model.
    ///
    /// You can parse the name and version to pass to subsequent [`KeyClient`] method calls.
    ///
    /// # Examples
    ///
    /// ```
    /// use azure_security_keyvault_keys::{models::{JsonWebKey, KeyBundle}, ResourceExt as _};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // KeyClient::get_key() will return a KeyBundle.
    /// let mut jwk = JsonWebKey::default();
    /// jwk.kid = Some("https://my-vault.vault.azure.net/keys/my-key/abcd1234?api-version=7.5".into());
    /// let mut key = KeyBundle::default();
    /// key.key = Some(jwk);
    ///
    /// let id = key.resource_id()?;
    /// assert_eq!(id.vault_url, "https://my-vault.vault.azure.net");
    /// assert_eq!(id.name, "my-key");
    /// assert_eq!(id.version, Some("abcd1234".into()));
    /// # Ok(())
    /// # }
    /// ```
    fn resource_id(&self) -> Result<ResourceId>;
}

impl<T> ResourceExt for T
where
    T: private::AsId,
{
    fn resource_id(&self) -> Result<ResourceId> {
        let Some(id) = self.as_id() else {
            return Err(azure_core::Error::message(
                ErrorKind::DataConversion,
                "missing resource id",
            ));
        };

        let url: Url = id.parse()?;
        deconstruct(url)
    }
}

fn deconstruct(url: Url) -> Result<ResourceId> {
    let vault_url = format!("{}://{}", url.scheme(), url.authority(),);
    let mut segments = url
        .path_segments()
        .ok_or_else(|| azure_core::Error::message(ErrorKind::DataConversion, "invalid url"))?;
    segments
        .next()
        .and_then(none_if_empty)
        .ok_or_else(|| azure_core::Error::message(ErrorKind::DataConversion, "missing collection"))
        .and_then(|col| {
            if col != "keys" {
                return Err(azure_core::Error::message(
                    ErrorKind::DataConversion,
                    "not in keys collection",
                ));
            }
            Ok(col)
        })?;
    let name = segments
        .next()
        .and_then(none_if_empty)
        .ok_or_else(|| azure_core::Error::message(ErrorKind::DataConversion, "missing name"))
        .map(String::from)?;
    let version = segments.next().and_then(none_if_empty).map(String::from);

    Ok(ResourceId {
        source_id: url.as_str().into(),
        vault_url,
        name,
        version,
    })
}

fn none_if_empty(s: &str) -> Option<&str> {
    if s.is_empty() {
        return None;
    }

    Some(s)
}

mod private {
    use crate::models::{DeletedKeyBundle, DeletedKeyItem, KeyBundle, KeyItem};

    pub trait AsId {
        fn as_id(&self) -> Option<&String>;
    }

    impl AsId for KeyBundle {
        fn as_id(&self) -> Option<&String> {
            self.key.as_ref()?.kid.as_ref()
        }
    }

    impl AsId for KeyItem {
        fn as_id(&self) -> Option<&String> {
            self.kid.as_ref()
        }
    }

    impl AsId for DeletedKeyBundle {
        fn as_id(&self) -> Option<&String> {
            self.key.as_ref()?.kid.as_ref()
        }
    }

    impl AsId for DeletedKeyItem {
        fn as_id(&self) -> Option<&String> {
            self.kid.as_ref()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{JsonWebKey, KeyBundle};

    use super::*;

    #[test]
    fn test_deconstruct() {
        deconstruct("file:///tmp".parse().unwrap()).expect_err("cannot-be-base url");
        deconstruct("https://vault.azure.net/".parse().unwrap()).expect_err("missing collection");
        deconstruct("https://vault.azure.net/collection/".parse().unwrap())
            .expect_err("invalid collection");
        deconstruct("https://vault.azure.net/keys/".parse().unwrap()).expect_err("missing name");

        let url: Url = "https://vault.azure.net/keys/name".parse().unwrap();
        assert_eq!(
            deconstruct(url.clone()).unwrap(),
            ResourceId {
                source_id: url.to_string(),
                vault_url: "https://vault.azure.net".into(),
                name: "name".into(),
                version: None
            }
        );

        let url: Url = "https://vault.azure.net/keys/name/version".parse().unwrap();
        assert_eq!(
            deconstruct(url.clone()).unwrap(),
            ResourceId {
                source_id: url.to_string(),
                vault_url: "https://vault.azure.net".into(),
                name: "name".into(),
                version: Some("version".into()),
            }
        );

        let url: Url = "https://vault.azure.net:443/keys/name/version"
            .parse()
            .unwrap();
        assert_eq!(
            deconstruct(url.clone()).unwrap(),
            ResourceId {
                source_id: url.to_string(),
                vault_url: "https://vault.azure.net".into(),
                name: "name".into(),
                version: Some("version".into()),
            }
        );

        let url: Url = "https://vault.azure.net:8443/keys/name/version"
            .parse()
            .unwrap();
        assert_eq!(
            deconstruct(url.clone()).unwrap(),
            ResourceId {
                source_id: url.to_string(),
                vault_url: "https://vault.azure.net:8443".into(),
                name: "name".into(),
                version: Some("version".into()),
            }
        );
    }

    #[test]
    fn from_secret_bundle() {
        let mut key = KeyBundle {
            key: None,
            ..Default::default()
        };
        key.resource_id().expect_err("missing resource id");

        let mut jwk = JsonWebKey {
            kid: None,
            ..Default::default()
        };
        key.key = Some(jwk.clone());
        key.resource_id().expect_err("missing resource id");

        let url: Url = "https://vault.azure.net/keys/name/version".parse().unwrap();
        jwk.kid = Some(url.to_string());
        key.key = Some(jwk);
        assert_eq!(
            key.resource_id().unwrap(),
            ResourceId {
                source_id: url.to_string(),
                vault_url: "https://vault.azure.net".into(),
                name: "name".into(),
                version: Some("version".into()),
            }
        );
    }
}
