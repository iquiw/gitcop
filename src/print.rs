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
