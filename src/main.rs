#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::{App, AppSettings};
use clap::Arg;
use git::Repository;
use std::fs::{OpenOptions};
use std::io::Read;

mod git;

fn main() -> Result<()> {

    const APP_SETTINGS: &[AppSettings] = &[
        AppSettings::UnifiedHelpMessage,
        AppSettings::ColoredHelp,
        AppSettings::VersionlessSubcommands,
    ];

    let matches = App::new("Commit Spammer")
        .settings(APP_SETTINGS)
        .version(env!("CARGO_PKG_VERSION"))
        .author("Paul D. <paul.delafosse@protonmail.com>")
        .about("A stupid cli to make millions of commit in seconds on your github account and look smart on twitter")
        .arg(Arg::with_name("number")
            .takes_value(true)
            .long("number")
            .short("n")
            .required(true)
            .value_name("NUMBER")
            .help("number of commit to make in your repo")
        ).get_matches();

    let number = matches.value_of("number");
    let number = number.unwrap();
    let number = number.parse::<u64>()
        .map_err(|err|anyhow!("Failed to parse <number> : {}", err))?;


    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("the_file")?;

    let repo = Repository::open().map_err(|err| anyhow!("Could not open git repository {}", err))?;

    for n in 0..number {
        if n % 2 == 0 {
            std::fs::write("the_file", "What's stopping you from coding like this?").map_err(|err| anyhow!("Cannot write {}", err))?;
        } else {
            std::fs::write("the_file", "It is QUANTITY rather than quality that matters.").map_err(|err| anyhow!("Cannot write {}", err))?;
        }
        file.sync_all().map_err(|err| anyhow!("Cannot sync {}", err))?;
        repo.add_all().map_err(|err| anyhow!("Cannot add to git index {}", err))?;
        repo.commit("the commit".to_string()).map_err(|err| anyhow!("Cannot commit {}", err))?;
        println!("Created commit {}/{}", n, number);
    }

    Ok(())
}