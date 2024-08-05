use crate::error::Error;
use crate::model::{UpdateUserInfoRequest, UserInfo};
use std::sync::RwLock;

pub struct UserInfoStore {
    cache: RwLock<UserInfo>,
}

impl UserInfoStore {
    pub fn new(user_info: UserInfo) -> Self {
        Self {
            cache: RwLock::new(user_info),
        }
    }

    pub fn get(&self) -> Result<UserInfo, Error> {
        Ok(self.cache.read().unwrap_or_else(|p| p.into_inner()).clone())
    }

    pub fn update(&self, update_user_info_request: UpdateUserInfoRequest) -> Result<UserInfo, Error> {
        let mut lock = self.cache.write().unwrap_or_else(|p| p.into_inner());

        let custom_fields = match update_user_info_request.custom_fields {
            None => lock.custom_fields.clone(),
            custom_fields @ Some(_) => custom_fields,
        };

        // This will lead to a lot of unnecessary clones. This implementation rewards bug prevention
        // over performance.
        let user_info = UserInfo {
            sub: update_user_info_request.subject.unwrap_or(lock.sub.clone()),
            name: update_user_info_request.name.unwrap_or(lock.name.clone()),
            given_name: update_user_info_request.given_name.unwrap_or(lock.given_name.clone()),
            family_name: update_user_info_request.family_name.unwrap_or(lock.family_name.clone()),
            nickname: update_user_info_request.nickname.unwrap_or(lock.nickname.clone()),
            locale: update_user_info_request.locale.unwrap_or(lock.locale.clone()),
            gender: update_user_info_request.gender.unwrap_or(lock.gender.clone()),
            birthdate: update_user_info_request.birthdate.unwrap_or(lock.birthdate.clone()),
            email: update_user_info_request.email.unwrap_or(lock.email.clone()),
            email_verified: update_user_info_request.email_verified.unwrap_or(lock.email_verified),
            picture: update_user_info_request.picture.unwrap_or(lock.picture.clone()),
            updated_at: update_user_info_request.updated_at.unwrap_or(lock.updated_at),
            custom_fields,
        };

        *lock = user_info.clone();

        Ok(user_info)
    }

    pub fn replace(&self, user_info: UserInfo) -> Result<(), Error> {
        let mut lock = self.cache.write().unwrap_or_else(|p| p.into_inner());
        *lock = user_info;

        Ok(())
    }
}
