use clap::Parser;
use gen::Pattern;
use rand::thread_rng;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Pattern to generate
    pattern: String,
}

fn main() {
    let cli = Cli::parse();

    match Pattern::parse(&cli.pattern) {
        Ok(pattern) => print!("{}", pattern.generate(&mut thread_rng())),
        Err(err) => eprintln!("Couldn't parse pattern: {err}")
    }
}
