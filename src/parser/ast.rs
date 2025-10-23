use std::fmt::{self, Display, Formatter};

use crate::parser::token::Token;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Redirectee {
    FileName(String),
    FileDescriptor(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectionType {
    Stdin,
    Stdout,
    Stderr,
}

impl TryFrom<Token> for RedirectionType {
    type Error = String;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Langle => Ok(Self::Stdin),
            Token::Rangle2 | Token::Rangle2F | Token::DoubleRangle2 => Ok(Self::Stderr),
            Token::Rangle | Token::RangleF | Token::DoubleRangle => Ok(Self::Stdout),
            _ => Err("Invalid redirection marker".into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectionPermission {
    Truncate,
    Append,
    Standard,
}
impl TryFrom<Token> for RedirectionPermission {
    type Error = String;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Langle | Token::Rangle | Token::Rangle2 => Ok(Self::Standard),
            Token::Rangle2F | Token::RangleF => Ok(Self::Truncate),
            Token::DoubleRangle2 | Token::DoubleRangle => Ok(Self::Append),
            _ => Err("Invalid redirection marker".into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Redirection {
    pub redirectee: Redirectee,
    pub type_: RedirectionType,
    pub permissions: RedirectionPermission,
}

impl TryFrom<(Token, String)> for Redirection {
    type Error = String;
    fn try_from((token, s): (Token, String)) -> Result<Self, Self::Error> {
        let type_ = RedirectionType::try_from(token.clone())?;
        let permissions = RedirectionPermission::try_from(token)?;
        Ok(Self {
            type_,
            permissions,
            redirectee: Redirectee::FileName(s),
        })
    }
}

impl Redirection {
    pub const fn new(
        redirectee: Redirectee,
        type_: RedirectionType,
        permissions: RedirectionPermission,
    ) -> Self {
        Self {
            redirectee,
            type_,
            permissions,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
    pub background: bool,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.name)?;
        for arg in &self.args {
            write!(f, "{arg} ")?;
        }
        Ok(())
    }
}

impl Command {
    pub const fn new(
        name: String,
        args: Vec<String>,
        redirections: Vec<Redirection>,
        background: bool,
    ) -> Self {
        Self {
            name,
            args,
            redirections,
            background,
        }
    }
}
