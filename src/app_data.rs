use crate::config::Config;
use crate::error::Error;
use crate::model::{Issuer, Subject};
use crate::store::{
    AudiencesStore, AuthorizationsStore, CustomClaimsStore, JwksStore, LoginStatesStore, UserInfoStore, UsersStore,
};
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
    users: UsersStore,
    login_states: LoginStatesStore,
}

impl AppData {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let pinned = match (
            config.jwks().private_key_pem_path().as_deref(),
            config.jwks().kid().as_deref(),
        ) {
            (Some(path), Some(kid)) => {
                let pem = std::fs::read(path)?;
                Some((pem, kid.to_string()))
            }
            (None, None) => None,
            _ => {
                return Err(Error::Config(
                    "[jwks] must specify either both private_key_pem_path and kid, or neither".into(),
                ));
            }
        };

        Ok(Self {
            issuer: config.issuer().clone(),
            subject: config.subject().clone(),
            audiences: AudiencesStore::new(config.audience()),
            authorizations: AuthorizationsStore::default(),
            custom_claims: CustomClaimsStore::new(config.access_token()),
            jwks: JwksStore::new(pinned)?,
            user_info: UserInfoStore::new(config.user_info().into()),
            users: UsersStore::new(config.user()),
            login_states: LoginStatesStore::default(),
        })
    }
}
