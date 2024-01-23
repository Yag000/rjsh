use crate::shell::Shell;

use self::cd::Cd;
use self::exit::Exit;

mod cd;
mod exit;

pub trait BuiltIn {
    fn call(&self, shell: &mut dyn Shell, args: &[String]) -> anyhow::Result<i32>;
}

pub fn get_builtin(command: &crate::parser::ast::Command) -> Option<Box<dyn BuiltIn>> {
    match command.name.as_str() {
        "cd" => Some(Box::new(Cd {})),
        "exit" => Some(Box::new(Exit{})),
        _ => None,
    }
}
