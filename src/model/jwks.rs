use std::str::FromStr;
use std::sync::{RwLock, RwLockWriteGuard};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, KeyUsage, SubjectKeyIdentifier};
use openssl::x509::{X509NameBuilder, X509};
use rand::seq::SliceRandom;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

pub struct JwksStore {
    cache: RwLock<Jwks>,
}

impl JwksStore {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            cache: RwLock::new(Jwks::new()?),
        })
    }

    pub fn get(&self) -> Result<Jwks, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.cache.read().unwrap_or_else(|p| p.into_inner()).random_jwk()
    }

    pub fn rotate_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *jwks = jwks.rotate_keys()?;
        Ok(())
    }

    pub fn revoke_keys(&self) -> Result<(), Error> {
        let mut jwks: RwLockWriteGuard<Jwks> = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *jwks = jwks.revoke_keys()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            keys: (1..=3)
                .into_iter()
                .map(|_| Jwk::new(None))
                .collect::<Result<Vec<Jwk>, Error>>()?,
        })
    }

    pub fn find(&self, kid: String) -> Option<Jwk> {
        self.keys.iter().find(|jwk| jwk.kid == kid).cloned()
    }

    pub fn random_jwk(&self) -> Result<Jwk, Error> {
        self.keys
            .choose(&mut rand::thread_rng())
            .ok_or(Error::EmptyJwks)
            .map(|jwks| jwks.clone())
    }

    pub fn parse<T: DeserializeOwned>(&self, token: &str, audience: &[impl ToString]) -> Result<T, Error> {
        let header: Header = jsonwebtoken::decode_header(token)?;

        if let Some(jwk) = header.kid.and_then(|kid| self.find(kid)) {
            let mut validation: Validation = Validation::new(Algorithm::from_str(jwk.alg())?);

            if !audience.is_empty() {
                validation.set_audience(audience);
            }

            let decoding_key: DecodingKey = DecodingKey::from_rsa_components(jwk.modulus(), jwk.exponent())?;
            Ok(jsonwebtoken::decode(token, &decoding_key, &validation)?.claims)
        } else {
            Err(Error::JwtMissingKid)
        }
    }

    pub fn rotate_keys(&self) -> Result<Self, Error> {
        let mut keys: Vec<Jwk> = self.keys.clone();
        keys.insert(0, Jwk::new(None)?);
        keys.pop();
        Ok(Self { keys })
    }

    pub fn revoke_keys(&self) -> Result<Self, Error> {
        Jwks::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwk {
    kty: String,
    n: String,
    e: String,
    alg: String,
    kid: String,
    r#use: String,
    x5c: Vec<String>,
    #[serde(skip_serializing)]
    private_key_pem: Vec<u8>,
}

impl Jwk {
    pub fn new(rsa: Option<Rsa<Private>>) -> Result<Jwk, Error> {
        let rsa: Rsa<Private> = match rsa {
            Some(rsa) => rsa,
            None => Rsa::generate(2048)?,
        };

        let modulus: Vec<u8> = rsa.n().to_vec();
        let exponent: Vec<u8> = rsa.e().to_vec();
        let key_pair: PKey<Private> = PKey::from_rsa(rsa)?;

        let x509 = generate_x509_cert(&key_pair)?;
        let x509pem = x509.to_pem()?;
        let x509cert = String::from_utf8(x509pem)?
            .replace('\n', "")
            .replace("-----BEGIN CERTIFICATE-----", "")
            .replace("-----END CERTIFICATE-----", "");

        Ok(Self {
            kty: "RSA".to_string(),
            n: base64_url::encode(&modulus),
            e: base64_url::encode(&exponent),
            alg: "RS256".to_string(),
            kid: Uuid::new_v4().to_string(),
            r#use: "sig".to_string(),
            x5c: vec![x509cert],
            private_key_pem: key_pair.private_key_to_pem_pkcs8()?,
        })
    }

    pub fn encode<T: Serialize>(&self, t: &T) -> Result<String, Error> {
        let mut header: Header = Header::new(Algorithm::RS256);
        header.kid = Some(self.kid().to_string());
        let key: EncodingKey = EncodingKey::from_rsa_pem(self.private_key_pem())?;
        Ok(jsonwebtoken::encode(&header, &t, &key)?)
    }

    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub fn alg(&self) -> &str {
        &self.alg
    }

    pub fn modulus(&self) -> &str {
        &self.n
    }

    pub fn exponent(&self) -> &str {
        &self.e
    }

    pub fn private_key_pem(&self) -> &Vec<u8> {
        &self.private_key_pem
    }

    pub fn rsa(&self) -> Result<Rsa<Private>, Error> {
        Ok(Rsa::private_key_from_pem(&self.private_key_pem)?)
    }
}

fn generate_x509_cert(key_pair: &PKey<Private>) -> Result<X509, ErrorStack> {
    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "IT")?;
    x509_name.append_entry_by_text("O", "Prima CA")?;
    x509_name.append_entry_by_text("CN", "Prima CA")?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(KeyUsage::new().critical().key_cert_sign().crl_sign().build()?)?;

    let subject_key_identifier = SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok(cert)
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::model::ca::CA;
    use crate::model::jwks::JwksStore;
    use crate::model::{Claims, GrantType, Jwk, Jwks};

    #[test]
    fn its_possible_to_generate_jwks_and_parse_claims_using_given_jwks_test() {
        let ca = match CA::new() {
            Ok(ca) => ca,
            _ => panic!("Cannot initialize CA"),
        };

        let jwk_store: JwksStore = JwksStore::new(&ca).unwrap();
        let audience: &str = "audience";
        let permission: &str = "permission";
        let issuer: &str = "issuer";
        let gty: GrantType = GrantType::ClientCredentials;

        let jwks: Jwks = jwk_store.get().unwrap();
        let random_jwk: Jwk = jwks.random_jwk().unwrap();

        let claims: Claims = Claims::new(
            audience.to_string(),
            vec![permission.to_string()],
            issuer.to_string(),
            gty.clone(),
        );

        let jwt: String = random_jwk.encode(&claims).unwrap();

        let result: Result<Claims, Error> = jwks.parse(jwt.as_ref(), &[audience]);

        assert!(result.is_ok());
        let claims: Claims = result.unwrap();
        assert_eq!(claims.audience(), audience);
        assert!(claims.has_permission(permission));
        assert_eq!(claims.issuer(), issuer);
        assert_eq!(claims.grant_type().to_string(), gty.to_string());
    }
}
