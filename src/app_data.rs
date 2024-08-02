use crate::config::Config;
use crate::error::Error;
use crate::model::{Issuer, Subject};
use crate::store::{AudiencesStore, AuthorizationsStore, CustomClaimsStore, JwksStore, UserInfoStore};
use derive_getters::Getters;

#[derive(Getters)]
pub struct AppData {
    issuer: Issuer,
    subject: Subject,
    audiences: AudiencesStore,
    authorizations: AuthorizationsStore,
    custom_claims: CustomClaimsStore,
    jwks: JwksStore,
    user_info: UserInfoStore,
}

impl AppData {
    pub fn new(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            issuer: config.issuer().clone(),
            subject: config.subject().clone(),
            audiences: AudiencesStore::new(config.audience()),
            authorizations: AuthorizationsStore::default(),
            custom_claims: CustomClaimsStore::new(config.access_token()),
            jwks: JwksStore::new()?,
            user_info: UserInfoStore::new(config.user_info().into()),
        })
    }
}
