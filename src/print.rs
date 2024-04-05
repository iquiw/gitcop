use yansi::{Paint, Painted};

type Printable<'a> = Painted<&'a str>;

pub fn color_init() {
}

pub fn warn(s: &str) -> Printable<'_> {
    s.red()
}

pub fn good(s: &str) -> Printable<'_> {
    s.green()
}
