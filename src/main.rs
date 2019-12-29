

#![feature(pattern)]

use crate::stabilize::Stabilize;
use std::io::Write;
use clap::Clap;

pub mod file;
pub mod stabilize;

#[derive(Clap, Debug)]
#[clap(name = "stabilize")]
struct Cli {
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
    #[clap(short = "f", long = "feature", help = "the feature to stabilize")]
    feature: Option<String>,
}

fn main() {
    std::env::set_current_dir(std::path::Path::new("../rust")).expect("cannot set dir");

    let opt = Cli::parse();
    if let Some(feature) = opt.feature {
        Stabilize::try_new(&feature)
            .expect("issue with rustc path")
            .start()
            .expect("cannot stabilise");
    }
}

pub fn prompt_reply_stdout(prompt: &str) -> std::io::Result<String> {
    let mut reply = String::new();
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", prompt)?;
    stdout.flush()?;
    std::io::stdin().read_line(&mut reply)?;

    Ok(reply)
}
