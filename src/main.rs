use clap::Parser;
use wyag::{Cli, run};

fn main() {
    let cli = Cli::parse();
    run(cli);
}
