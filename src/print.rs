use ansi_term::Colour::{Green, Red};
use ansi_term::{self, ANSIGenericString};

type Printable<'a> = ANSIGenericString<'a, str>;

pub fn color_init() {
    #[cfg(windows)]
    let _ignore = ansi_term::enable_ansi_support();
}

pub fn warn<'a>(s: &'a str) -> Printable<'a> {
    Red.paint(s)
}

pub fn good<'a>(s: &'a str) -> Printable<'a> {
    Green.paint(s)
}

#[macro_export]
macro_rules! locked_print {
    ($($arg:tt)*) => {
        use std::io::{stdout, Write};
        let stdout = stdout();
        let mut handle = stdout.lock();
        write!(&mut handle, $($arg)*).unwrap();
    };
}

#[macro_export]
macro_rules! locked_println {
    ($($arg:tt)*) => {
        use std::io::{stdout, Write};
        let stdout = stdout();
        let mut handle = stdout.lock();
        writeln!(&mut handle, $($arg)*).unwrap();
    };
}
