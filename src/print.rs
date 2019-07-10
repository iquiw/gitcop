use yansi::Paint;

type Printable<'a> = Paint<&'a str>;

pub fn color_init() {
    #[cfg(windows)]
    let _ignore = Paint::enable_windows_ascii();
}

pub fn warn<'a>(s: &'a str) -> Printable<'a> {
    Paint::red(s)
}

pub fn good<'a>(s: &'a str) -> Printable<'a> {
    Paint::green(s)
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
