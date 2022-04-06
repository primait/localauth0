use crate::model::Jwt;

#[derive(Debug)]
pub enum Msg {
    GenerateToken,
    TokenReceived(Option<Jwt>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
}
