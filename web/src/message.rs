#[derive(Debug)]
pub enum Msg {
    UpdateAudience,
    GenerateToken,
    TokenReceived(Option<String>),
    AddPermission,
    RemovePermission(String),
    SetPermissions,
}
