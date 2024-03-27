use std::process::exit;

use clap::Parser;
use gen::Pattern;
use rand::thread_rng;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Pattern to generate
    pattern: String,
    /// Don't output a trailing newline
    #[clap(short)]
    no_newline: bool,
}

fn main() {
    let cli = Cli::parse();
    let string = match Pattern::parse(&cli.pattern) {
        Ok(pattern) => pattern.generate(&mut thread_rng()),
        Err(e) => {
            eprintln!("Couldn't parse pattern: {e}");
            exit(-1)
        }
    };

    match cli.no_newline {
        false => println!("{string}"),
        true => print!("{string}")
    }
}
