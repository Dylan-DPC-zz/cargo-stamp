#![feature(pattern)]

use structopt::StructOpt;
use std::io::Write;
use crate::stabilize::Stabilize;

pub mod stabilize;
pub mod file;

#[derive(StructOpt, Debug)]
#[structopt(name = "stabilize")]
struct Cli {
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
    #[structopt(short = "f", long = "feature", help = "the feature to stabilize")]
    feature: Option<String>
}

fn main() {
    std::env::set_current_dir(std::path::Path::new("/home/dpc/Code/rust")).expect("cannot set dir");

    let opt = Cli::from_args();
    if let Some(feature) = opt.feature {
        Stabilize::try_new(&feature).expect("issue with rustc path").start().expect("cannot stabilise");

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