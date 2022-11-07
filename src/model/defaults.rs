use chrono::{DateTime, Utc};

const ISSUER: &str = "https://prima.localauth0.com/";
const USER_INFO_SUBJECT: &str = "google-apps|developers@prima.it";
const USER_INFO_NAME: &str = "Local";
const USER_INFO_NICKNAME: &str = "locie.auth0";
const USER_INFO_GIVEN_NAME: &str = "Locie";
const USER_INFO_FAMILY_NAME: &str = "Auth0";
const USER_INFO_LOCALE: &str = "en";
const USER_INFO_GENDER: &str = "none";
const USER_INFO_BIRTHDATE: &str = "2022-02-11";
const USER_INFO_EMAIL: &str = "developers@prima.it";
const USER_INFO_UPDATED_AT: &str = "2022-11-11T11:00:00Z";
const USER_INFO_EMAIL_VERIFIED: bool = true;
const USER_INFO_PICTURE: &str = "https://github.com/primait/localauth0/blob/6f71c9318250219a9d03fb72afe4308b8824aef7/web/assets/static/media/localauth0.png";

const HTTP_PORT: u16 = 3000;
const HTTPS_PORT: u16 = 3001;

pub fn issuer() -> String {
    ISSUER.to_string()
}

pub fn user_info_subject() -> String {
    USER_INFO_SUBJECT.to_string()
}

pub fn user_info_name() -> String {
    USER_INFO_NAME.to_string()
}

pub fn user_info_given_name() -> String {
    USER_INFO_GIVEN_NAME.to_string()
}

pub fn user_info_family_name() -> String {
    USER_INFO_FAMILY_NAME.to_string()
}

pub fn user_info_nickname() -> String {
    USER_INFO_NICKNAME.to_string()
}

pub fn user_info_locale() -> String {
    USER_INFO_LOCALE.to_string()
}

pub fn user_info_gender() -> String {
    USER_INFO_GENDER.to_string()
}

pub fn user_info_birthdate() -> String {
    USER_INFO_BIRTHDATE.to_string()
}

pub fn user_info_email() -> String {
    USER_INFO_EMAIL.to_string()
}

pub fn user_info_email_verified() -> bool {
    USER_INFO_EMAIL_VERIFIED
}

pub fn user_info_updated_at() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(USER_INFO_UPDATED_AT).unwrap().into()
}

pub fn user_info_picture() -> String {
    USER_INFO_PICTURE.to_string()
}

pub fn http_port() -> u16 {
    HTTP_PORT
}

pub fn https_port() -> u16 {
    HTTPS_PORT
}
