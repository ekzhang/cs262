//! Solutions to CS 262 assignments in Rust.

#![forbid(unsafe_code)]

use clap::Parser;

pub mod wire;

/// Command-line interface for CS 262 solutions.
#[derive(Parser, Debug)]
pub enum Cli {
    #[clap(subcommand)]
    Wire(Wire),
}

#[derive(Parser, Debug)]
pub enum Wire {
    Client,
    Server,
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Cli::Wire(Wire::Client) => wire::run_client()?,
            Cli::Wire(Wire::Server) => wire::run_server()?,
        }
        Ok(())
    }
}
