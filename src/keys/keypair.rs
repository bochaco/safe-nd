// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Module providing keys, keypairs, and signatures.
//!
//! The easiest way to get a `PublicKey` is to create a random `Keypair` first through one of the
//! `new` functions. A `PublicKey` can't be generated by itself; it must always be derived from a
//! secret key.

use crate::{Error, Result};
use crate::{PublicKey, SecretKey, Signature, SignatureShare};

use ed25519_dalek::Signer;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};
use threshold_crypto::{self, serde_impl::SerdeSecret};

/// Wrapper for different keypair types.
#[derive(Serialize, Deserialize)]
pub enum Keypair {
    /// Ed25519 keypair.
    Ed25519(ed25519_dalek::Keypair),
    /// BLS keypair.
    Bls(BlsKeypair),
    /// BLS keypair share.
    BlsShare(BlsKeypairShare),
}

impl Debug for Keypair {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Keypair::")?;
        match self {
            Self::Ed25519(_) => write!(formatter, "Ed25519(..)"),
            Self::Bls(_) => write!(formatter, "Bls(..)"),
            Self::BlsShare(_) => write!(formatter, "BlsShare(..)"),
        }
    }
}

// Need to manually implement this due to a missing impl in `Ed25519::Keypair`.
impl PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ed25519(keypair), Self::Ed25519(other_keypair)) => {
                // TODO: After const generics land, remove the `to_vec()` calls.
                keypair.to_bytes().to_vec() == other_keypair.to_bytes().to_vec()
            }
            (Self::Bls(keypair), Self::Bls(other_keypair)) => keypair == other_keypair,
            (Self::BlsShare(keypair), Self::BlsShare(other_keypair)) => keypair == other_keypair,
            _ => false,
        }
    }
}

// Need to manually implement this due to a missing impl in `Ed25519::Keypair`.
impl Eq for Keypair {}

impl Keypair {
    /// Constructs a random Ed25519 public keypair.
    pub fn new_ed25519<T: CryptoRng + Rng>(rng: &mut T) -> Self {
        let keypair = ed25519_dalek::Keypair::generate(rng);
        Self::Ed25519(keypair)
    }

    /// Constructs a random BLS public keypair.
    pub fn new_bls<T: CryptoRng + Rng>(rng: &mut T) -> Self {
        let bls_secret_key: threshold_crypto::SecretKey = rng.gen();
        let bls_public_key = bls_secret_key.public_key();
        let keypair = BlsKeypair {
            secret: SerdeSecret(bls_secret_key),
            public: bls_public_key,
        };
        Self::Bls(keypair)
    }

    /// Constructs a BLS public keypair share.
    pub fn new_bls_share(
        index: usize,
        secret_share: threshold_crypto::SecretKeyShare,
        public_key_set: threshold_crypto::PublicKeySet,
    ) -> Self {
        let public_share = secret_share.public_key_share();
        let keypair_share = BlsKeypairShare {
            index,
            secret: SerdeSecret(secret_share),
            public: public_share,
            public_key_set,
        };
        Self::BlsShare(keypair_share)
    }

    /// Returns the public key associated with this keypair.
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Ed25519(keypair) => PublicKey::Ed25519(keypair.public),
            Self::Bls(keypair) => PublicKey::Bls(keypair.public),
            Self::BlsShare(keypair) => PublicKey::BlsShare(keypair.public),
        }
    }

    /// Returns the secret key associated with this keypair.
    pub fn secret_key(&self) -> Result<SecretKey> {
        match self {
            Self::Ed25519(keypair) => {
                let bytes = keypair.secret.to_bytes();
                match ed25519_dalek::SecretKey::from_bytes(&bytes) {
                    Ok(sk) => Ok(SecretKey::Ed25519(sk)),
                    Err(_) => Err(Error::Unexpected(
                        "Could not deserialise Ed25519 secret key".to_string(),
                    )),
                }
            }
            Self::Bls(keypair) => Ok(SecretKey::Bls(keypair.secret.clone())),
            Self::BlsShare(keypair) => Ok(SecretKey::BlsShare(keypair.secret.clone())),
        }
    }

    /// Signs with the underlying keypair.
    pub fn sign(&self, data: &[u8]) -> Signature {
        match self {
            Self::Ed25519(keypair) => Signature::Ed25519(keypair.sign(&data)),
            Self::Bls(keypair) => Signature::Bls(keypair.secret.sign(data)),
            Self::BlsShare(keypair) => {
                let index = keypair.index;
                let share = keypair.secret.sign(data);
                Signature::BlsShare(SignatureShare { index, share })
            }
        }
    }
}

impl From<threshold_crypto::SecretKey> for Keypair {
    fn from(sk: threshold_crypto::SecretKey) -> Self {
        let public = sk.public_key();
        let keypair = BlsKeypair {
            secret: SerdeSecret(sk),
            public,
        };
        Self::Bls(keypair)
    }
}

impl From<&threshold_crypto::SecretKey> for Keypair {
    fn from(sk: &threshold_crypto::SecretKey) -> Self {
        let public = sk.public_key();
        let keypair = BlsKeypair {
            secret: SerdeSecret(sk.clone()),
            public,
        };
        Self::Bls(keypair)
    }
}

impl From<SerdeSecret<threshold_crypto::SecretKey>> for Keypair {
    fn from(sk: SerdeSecret<threshold_crypto::SecretKey>) -> Self {
        let public = sk.public_key();
        let keypair = BlsKeypair { secret: sk, public };
        Self::Bls(keypair)
    }
}

impl From<&SerdeSecret<threshold_crypto::SecretKey>> for Keypair {
    fn from(sk: &SerdeSecret<threshold_crypto::SecretKey>) -> Self {
        let public = sk.public_key();
        let keypair = BlsKeypair {
            secret: sk.clone(),
            public,
        };
        Self::Bls(keypair)
    }
}

impl From<ed25519_dalek::SecretKey> for Keypair {
    fn from(secret: ed25519_dalek::SecretKey) -> Self {
        let public: ed25519_dalek::PublicKey = (&secret).into();

        let keypair = ed25519_dalek::Keypair { public, secret };

        Self::Ed25519(keypair)
    }
}

/// BLS keypair.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlsKeypair {
    /// Secret key.
    pub secret: SerdeSecret<threshold_crypto::SecretKey>,
    /// Public key.
    pub public: threshold_crypto::PublicKey,
}

/// BLS keypair share.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlsKeypairShare {
    /// Share index.
    pub index: usize,
    /// Secret key share.
    pub secret: SerdeSecret<threshold_crypto::SecretKeyShare>,
    /// Public key share.
    pub public: threshold_crypto::PublicKeyShare,
    /// Public key set. Necessary for producing proofs.
    pub public_key_set: threshold_crypto::PublicKeySet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use bincode::deserialize as deserialise;

    fn gen_keypairs() -> Vec<Keypair> {
        let mut rng = rand::thread_rng();
        let bls_secret_key = threshold_crypto::SecretKeySet::random(1, &mut rng);
        vec![
            Keypair::new_ed25519(&mut rng),
            Keypair::new_bls(&mut rng),
            Keypair::new_bls_share(
                0,
                bls_secret_key.secret_key_share(0),
                bls_secret_key.public_keys(),
            ),
        ]
    }

    // Test serialising and deserialising key pairs.
    #[test]
    fn serialisation_key_pair() -> Result<()> {
        let keypairs = gen_keypairs();

        for keypair in keypairs {
            let encoded = utils::serialise(&keypair);
            let decoded: Keypair = deserialise(&encoded).map_err(|_| "Error deserialising key")?;

            assert_eq!(decoded, keypair);
        }

        Ok(())
    }
}
