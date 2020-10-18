use yansi::Paint;

type Printable<'a> = Paint<&'a str>;

pub fn color_init() {
    #[cfg(windows)]
    let _ignore = Paint::enable_windows_ascii();
}

pub fn warn(s: &str) -> Printable<'_> {
    Paint::red(s)
}

pub fn good(s: &str) -> Printable<'_> {
    Paint::green(s)
}
