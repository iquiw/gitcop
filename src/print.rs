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
