use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String),

    Langle,
    Rangle,
    RangleF,
    DoubleRangle,
    Rangle2,
    Rangle2F,
    DoubleRangle2,

    And,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::String(s) => write!(f, "{}", s),
            Token::Langle => write!(f, "<"),
            Token::Rangle => write!(f, ">"),
            Token::RangleF => write!(f, ">|"),
            Token::DoubleRangle => write!(f, ">>"),
            Token::Rangle2 => write!(f, "2>"),
            Token::Rangle2F => write!(f, "2>|"),
            Token::DoubleRangle2 => write!(f, "2>>"),
            Token::And => write!(f, "&"),
        }
    }
}
