use std::fmt::Display;
use std::io::Write;
use std::process;

#[macro_export]
macro_rules! stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

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
            stderr!("{}: {}", msg, e);
            process::exit(1)
        })
    }
}
