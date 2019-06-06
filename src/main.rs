use clap::clap_app;

use gitcop::cmd;
use gitcop::config;

fn main() {
    let matches = clap_app!( myapp =>
        (name: "gitcop")
        (@subcommand sync =>
         (about: "Sync repos")
         (@arg REPO: ... "Name of repos"))
    )
    .get_matches();

    let cfg = config::load_config(".gitcop.toml").unwrap();
    match matches.subcommand() {
        ("sync", Some(sub_m)) => {
            let names = sub_m
                .values_of("REPO")
                .map(|vs| vs.collect())
                .unwrap_or(vec![]);
            if let Err(err) = cmd::sync(&cfg, &names) {
                eprintln!("gitcop: sync failed, Error: {}", err);
            }
        }
        _ => eprintln!("{}", matches.usage()),
    }
}
