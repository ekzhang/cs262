use clap::Parser;
use cs262::Cli;

fn main() -> anyhow::Result<()> {
    Cli::parse().run()
}
