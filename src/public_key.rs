// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{Ed25519Digest, Error, XorName, XOR_NAME_LEN};
use ed25519_dalek;
use serde::{Deserialize, Serialize};
use threshold_crypto;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum PublicKey {
    Ed25519(ed25519_dalek::PublicKey),
    Bls(threshold_crypto::PublicKey),
    BlsShare(threshold_crypto::PublicKeyShare),
}

impl PublicKey {
    pub fn verify_detached<T: AsRef<[u8]>>(
        &self,
        signature: &Signature,
        data: T,
    ) -> Result<(), Error> {
        let is_valid = match (self, signature) {
            (PublicKey::Ed25519(pub_key), Signature::Ed25519(sig)) => {
                pub_key.verify::<Ed25519Digest>(data.as_ref(), sig).is_ok()
            }
            (PublicKey::Bls(pub_key), Signature::Bls(sig)) => pub_key.verify(sig, data),
            (PublicKey::BlsShare(pub_key), Signature::BlsShare(sig)) => pub_key.verify(sig, data),
            _ => return Err(Error::SigningKeyTypeMismatch),
        };
        if is_valid {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }
}

impl From<PublicKey> for XorName {
    fn from(public_key: PublicKey) -> Self {
        let bytes = match public_key {
            PublicKey::Ed25519(pub_key) => {
                return XorName(pub_key.to_bytes());
            }
            PublicKey::Bls(pub_key) => pub_key.to_bytes(),
            PublicKey::BlsShare(pub_key) => pub_key.to_bytes(),
        };
        let mut xor_name = XorName::default();
        xor_name.0.clone_from_slice(&bytes[..XOR_NAME_LEN]);
        xor_name
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum Signature {
    Ed25519(ed25519_dalek::Signature),
    Bls(threshold_crypto::Signature),
    BlsShare(threshold_crypto::SignatureShare),
}