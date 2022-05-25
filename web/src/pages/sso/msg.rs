use crate::pages::model::Jwt;

#[derive(Debug)]
pub enum Msg {
    TokenReceived(Jwt),
}
