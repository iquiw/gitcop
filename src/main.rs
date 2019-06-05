use gitcop::cmd;
use gitcop::config;

fn main() {
    let cfg = config::load_config(".gitcop.toml").unwrap();
    if let Err(err) = cmd::sync(&cfg) {
        eprintln!("gitcop: sync failed, Error: {}", err);
    }
}
