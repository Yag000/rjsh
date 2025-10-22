pub trait UnwrapPrintError {
    type T;
    fn unwrap_error_with_print(self) -> Self::T;
}

impl UnwrapPrintError for Result<i32, anyhow::Error> {
    type T = i32;
    fn unwrap_error_with_print(self) -> i32 {
        match self {
            Ok(value) => value,
            Err(error) => {
                eprintln!("rjsh: {error}");
                1
            }
        }
    }
}
