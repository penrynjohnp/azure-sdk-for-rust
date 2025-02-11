// Copyright (c) Microsoft Corporation. All rights reserved.
//
// Licensed under the MIT License. See License.txt in the project root for license information.
// Code generated by Microsoft (R) Rust Code Generator. DO NOT EDIT.

mod generated;
mod resource;

pub mod clients {
    pub use crate::generated::clients::*;
}

pub mod models {
    pub use crate::generated::enums::*;
    pub use crate::generated::models::*;
}

pub use crate::generated::clients::{
    KeyClient, KeyClientBackupKeyOptions, KeyClientCreateKeyOptions, KeyClientDecryptOptions,
    KeyClientDeleteKeyOptions, KeyClientEncryptOptions, KeyClientGetDeletedKeyOptions,
    KeyClientGetDeletedKeysOptions, KeyClientGetKeyOptions, KeyClientGetKeyRotationPolicyOptions,
    KeyClientGetKeyVersionsOptions, KeyClientGetKeysOptions, KeyClientGetRandomBytesOptions,
    KeyClientImportKeyOptions, KeyClientOptions, KeyClientPurgeDeletedKeyOptions,
    KeyClientRecoverDeletedKeyOptions, KeyClientReleaseOptions, KeyClientRestoreKeyOptions,
    KeyClientRotateKeyOptions, KeyClientSignOptions, KeyClientUnwrapKeyOptions,
    KeyClientUpdateKeyOptions, KeyClientUpdateKeyRotationPolicyOptions, KeyClientVerifyOptions,
    KeyClientWrapKeyOptions,
};

pub use resource::*;
