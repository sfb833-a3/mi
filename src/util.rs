use std::fmt::Display;
use std::process;

pub trait OrExit {
    type RetVal;

    fn or_exit(self, msg: &str) -> Self::RetVal;
}

impl<T, E> OrExit for Result<T, E>
where
    E: Display,
{
    type RetVal = T;

    fn or_exit(self, msg: &str) -> Self::RetVal {
        self.unwrap_or_else(|e: E| -> T {
            eprintln!("{}: {}", msg, e);
            process::exit(1)
        })
    }
}
