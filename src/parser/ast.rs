use std::fmt::{self, Display, Formatter};

use crate::lexer::token::Token;

#[derive(Debug, PartialEq)]
pub enum Redirectee {
    FileName(String),
    FileDescriptor(u32),
}

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    Stdin,
    Stdout,
    Stderr,
}

impl TryFrom<Token> for RedirectionType {
    type Error = String;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Langle => Ok(RedirectionType::Stdin),
            Token::Rangle2 | Token::Rangle2F | Token::DoubleRangle2 => Ok(RedirectionType::Stderr),
            Token::Rangle | Token::RangleF | Token::DoubleRangle => Ok(RedirectionType::Stdout),
            _ => Err("Invalid redirection marker".into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RedirectionPermission {
    Destroy,
    Append,
    Standard,
}
impl TryFrom<Token> for RedirectionPermission {
    type Error = String;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Langle | Token::Rangle | Token::Rangle2 => Ok(RedirectionPermission::Standard),
            Token::Rangle2F | Token::RangleF => Ok(RedirectionPermission::Destroy),
            Token::DoubleRangle2 | Token::DoubleRangle => Ok(RedirectionPermission::Append),
            _ => Err("Invalid redirection marker".into()),
        }
    }
}

#[derive(Debug, PartialEq)]
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
        Ok(Redirection {
            type_,
            permissions,
            redirectee: Redirectee::FileName(s),
        })
    }
}

impl Redirection {
    pub fn new(
        redirectee: Redirectee,
        type_: RedirectionType,
        permissions: RedirectionPermission,
    ) -> Self {
        Redirection {
            type_,
            permissions,
            redirectee,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for arg in &self.args {
            write!(f, "{} ", arg)?;
        }
        Ok(())
    }
}

impl Command {
    pub fn new(name: String, args: Vec<String>, redirections: Vec<Redirection>) -> Command {
        Command {
            name,
            args,
            redirections,
        }
    }
}
