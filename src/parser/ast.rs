use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Command {
    pub args: Vec<String>,
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
    pub fn new(args: Vec<String>) -> Command {
        Command { args }
    }
}
