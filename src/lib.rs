//! Solutions to CS 262 assignments in Rust.

#![forbid(unsafe_code)]

use clap::Parser;

pub mod lamport;
pub mod wire;

/// Command-line interface for CS 262 solutions.
#[derive(Parser, Debug)]
pub enum Cli {
    /// Assignment 1: Wire Protocols
    #[clap(subcommand)]
    Wire(Wire),

    /// Assignment 2: Scale Models and Logical Clocks
    Lamport,
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
            Cli::Lamport => lamport::run(),
        }
        Ok(())
    }
}
