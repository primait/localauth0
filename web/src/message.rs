#[derive(Debug)]
pub enum Msg {
    AddPermission,
    GenerateToken,
    TokenReceived(Option<String>),
}
