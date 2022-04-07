use crate::model::Jwt;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Msg {
    AudienceFocusOut,
    GenerateToken,
    TokenReceived(Option<Jwt>),
    ShowPermissions(HashSet<String>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
}
