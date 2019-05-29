// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT> or the Modified
// BSD license <LICENSE-BSD or https://opensource.org/licenses/BSD-3-Clause>,
// at your option. This file may not be copied, modified, or distributed
// except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use
// of the SAFE Network Software.

use crate::{XorName, XOR_NAME_LEN};
use threshold_crypto::{PublicKey, PK_SIZE};
use tiny_keccak;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct UnpubImmutableData {
    /// Contained ImmutableData.
    data: Vec<u8>,
    /// Contains a set of owners of this data. DataManagers enforce that a
    /// DELETE or OWNED-GET type of request is coming from the
    /// MaidManager Authority of the owners.
    owners: PublicKey,
}

impl UnpubImmutableData {
    /// Name.
    pub fn name(&self) -> XorName {
        // TODO: Use low-level arrays or slices instead of Vec.
        let mut bytes = Vec::with_capacity(XOR_NAME_LEN + PK_SIZE);
        bytes.extend_from_slice(&tiny_keccak::sha3_256(&self.data));
        bytes.extend_from_slice(&self.owners.to_bytes());
        tiny_keccak::sha3_256(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use threshold_crypto::SecretKey;

    #[test]
    fn deterministic_name() {
        let data1 = b"Hello".to_vec();
        let data2 = b"Goodbye".to_vec();

        let owner1 = SecretKey::random().public_key();
        let owner2 = SecretKey::random().public_key();

        let idata1 = UnpubImmutableData {
            data: data1.clone(),
            owners: owner1,
        };
        let idata2 = UnpubImmutableData {
            data: data1,
            owners: owner2,
        };
        let idata3 = UnpubImmutableData {
            data: data2,
            owners: owner1,
        };

        assert_eq!(idata1.name(), idata1.name());
        assert_eq!(idata2.name(), idata2.name());
        assert_eq!(idata3.name(), idata3.name());

        assert_ne!(idata1.name(), idata2.name());
        assert_ne!(idata1.name(), idata3.name());
        assert_ne!(idata2.name(), idata3.name());
    }
}
