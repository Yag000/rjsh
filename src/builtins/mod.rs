use self::cd::Cd;

mod cd;

pub trait BuiltIn {
    fn call(&self, args: &[String]) -> anyhow::Result<i32>;
}

pub fn get_builtin(command: &crate::parser::ast::Command) -> Option<Box<dyn BuiltIn>> {
    match command.name.as_str() {
        "cd" => Some(Box::new(Cd {})),
        _ => None,
    }
}
