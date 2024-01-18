#[derive(Debug, PartialEq)]
pub enum Token {
    String(String),
    EOF,
}
