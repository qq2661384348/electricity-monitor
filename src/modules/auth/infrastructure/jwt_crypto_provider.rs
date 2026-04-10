use std::sync::Once;

use hmac::{Hmac, Mac};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey,
    crypto::{CryptoProvider, JwkUtils, JwtSigner, JwtVerifier},
    errors::{Error, ErrorKind, Result},
    signature::{self, Signer, Verifier},
};
use sha2::{Sha256, Sha384, Sha512};

type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

macro_rules! define_hmac_signer {
    ($name:ident, $alg:expr, $hmac_type:ty) => {
        struct $name($hmac_type);

        impl $name {
            fn new(encoding_key: &EncodingKey) -> Result<Self> {
                let inner = <$hmac_type>::new_from_slice(encoding_key.try_get_hmac_secret()?)
                    .map_err(|_| ErrorKind::InvalidKeyFormat)?;
                Ok(Self(inner))
            }
        }

        impl Signer<Vec<u8>> for $name {
            fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
                let mut signer = self.0.clone();
                signer.update(msg);
                Ok(signer.finalize().into_bytes().to_vec())
            }
        }

        impl JwtSigner for $name {
            fn algorithm(&self) -> Algorithm {
                $alg
            }
        }
    };
}

macro_rules! define_hmac_verifier {
    ($name:ident, $alg:expr, $hmac_type:ty) => {
        struct $name($hmac_type);

        impl $name {
            fn new(decoding_key: &DecodingKey) -> Result<Self> {
                let inner = <$hmac_type>::new_from_slice(decoding_key.try_get_hmac_secret()?)
                    .map_err(|_| ErrorKind::InvalidKeyFormat)?;
                Ok(Self(inner))
            }
        }

        impl Verifier<Vec<u8>> for $name {
            fn verify(
                &self,
                msg: &[u8],
                signature: &Vec<u8>,
            ) -> std::result::Result<(), signature::Error> {
                let mut verifier = self.0.clone();
                verifier.update(msg);
                verifier
                    .verify_slice(signature)
                    .map_err(signature::Error::from_source)
            }
        }

        impl JwtVerifier for $name {
            fn algorithm(&self) -> Algorithm {
                $alg
            }
        }
    };
}

define_hmac_signer!(Hs256Signer, Algorithm::HS256, HmacSha256);
define_hmac_signer!(Hs384Signer, Algorithm::HS384, HmacSha384);
define_hmac_signer!(Hs512Signer, Algorithm::HS512, HmacSha512);

define_hmac_verifier!(Hs256Verifier, Algorithm::HS256, HmacSha256);
define_hmac_verifier!(Hs384Verifier, Algorithm::HS384, HmacSha384);
define_hmac_verifier!(Hs512Verifier, Algorithm::HS512, HmacSha512);

fn new_signer(algorithm: &Algorithm, key: &EncodingKey) -> Result<Box<dyn JwtSigner>> {
    match algorithm {
        Algorithm::HS256 => Ok(Box::new(Hs256Signer::new(key)?)),
        Algorithm::HS384 => Ok(Box::new(Hs384Signer::new(key)?)),
        Algorithm::HS512 => Ok(Box::new(Hs512Signer::new(key)?)),
        _ => Err(Error::from(ErrorKind::InvalidAlgorithm)),
    }
}

fn new_verifier(algorithm: &Algorithm, key: &DecodingKey) -> Result<Box<dyn JwtVerifier>> {
    match algorithm {
        Algorithm::HS256 => Ok(Box::new(Hs256Verifier::new(key)?)),
        Algorithm::HS384 => Ok(Box::new(Hs384Verifier::new(key)?)),
        Algorithm::HS512 => Ok(Box::new(Hs512Verifier::new(key)?)),
        _ => Err(Error::from(ErrorKind::InvalidAlgorithm)),
    }
}

static HMAC_ONLY_PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: new_signer,
    verifier_factory: new_verifier,
    jwk_utils: JwkUtils::new_unimplemented(),
};

static INSTALL_HMAC_ONLY_PROVIDER: Once = Once::new();

pub fn ensure_jwt_crypto_provider() {
    INSTALL_HMAC_ONLY_PROVIDER.call_once(|| {
        HMAC_ONLY_PROVIDER
            .install_default()
            .expect("HMAC-only JWT provider should install exactly once");
    });
}
