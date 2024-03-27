use std::process::exit;

use clap::Parser;
use gen::Pattern;
use nom::{error::convert_error, Err};
use rand::thread_rng;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Pattern to generate
    pattern: String,
    /// Don't output a trailing newline
    #[clap(short)]
    no_newline: bool,
    /// Output debugging information
    #[clap(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();
    let pattern = match Pattern::parse(&cli.pattern) {
        Ok(pattern) => pattern,
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            eprintln!("{}", convert_error(cli.pattern.as_str(), e));
            exit(-1)
        }
        Err(Err::Incomplete(..)) => unreachable!()
    };

    if cli.debug {
        println!("{pattern:?}");
    }

    let string = pattern.generate(&mut thread_rng());

    match cli.no_newline {
        false => println!("{string}"),
        true => print!("{string}")
    }
}
