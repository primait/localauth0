use std::collections::HashSet;

use crate::pages::model::Jwt;

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
