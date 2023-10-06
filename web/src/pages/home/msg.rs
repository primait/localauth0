use std::collections::HashSet;
use yew::KeyboardEvent;

use crate::pages::model::Jwt;

#[derive(Debug)]
pub enum Msg {
    AudienceFocusOut,
    GenerateToken,
    CopyToken,
    TokenCopied,
    CopyFailed,
    ResetCopyButton,
    TokenReceived(Jwt),
    ShowPermissions(HashSet<String>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
    PermissionKeyUp(KeyboardEvent),
}
