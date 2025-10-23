use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
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
            Self::String(s) => write!(f, "{s}"),
            Self::Langle => write!(f, "<"),
            Self::Rangle => write!(f, ">"),
            Self::RangleF => write!(f, ">|"),
            Self::DoubleRangle => write!(f, ">>"),
            Self::Rangle2 => write!(f, "2>"),
            Self::Rangle2F => write!(f, "2>|"),
            Self::DoubleRangle2 => write!(f, "2>>"),
            Self::And => write!(f, "&"),
        }
    }
}

impl From<&str> for Token {
    fn from(s: &str) -> Self {
        match s {
            "<" => Self::Langle,
            ">" => Self::Rangle,
            ">|" => Self::RangleF,
            ">>" => Self::DoubleRangle,
            "2>" => Self::Rangle2,
            "2>|" => Self::Rangle2F,
            "2>>" => Self::DoubleRangle2,
            "&" => Self::And,
            _ => Self::String(s.to_string()),
        }
    }
}
