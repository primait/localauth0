use crate::model::Jwt;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Msg {
    AudienceFocusOut,
    GenerateToken,
    CopyToken,
    TokenCopied,
    CopyFailed,
    ResetCopyButton,
    TokenReceived(Option<Jwt>),
    ShowPermissions(HashSet<String>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
}
