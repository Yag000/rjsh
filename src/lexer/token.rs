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
