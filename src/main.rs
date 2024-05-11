use clap::Parser;
use img_sort::arguments::Arguments;
use std::process;


fn main() {
    // Parse the arguments
    let args = Arguments::parse();

    // Validate args to make config
    let config = Arguments::validate(&args).unwrap_or_else(|err| {
        eprintln!("Problem validating arguments: {err}");
        process::exit(1)
    });

    if let Err(e) = img_sort::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
