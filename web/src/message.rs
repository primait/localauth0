use crate::model::Jwt;

#[derive(Debug)]
pub enum Msg {
    UpdateAudience,
    GenerateToken,
    TokenReceived(Option<Jwt>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
}
