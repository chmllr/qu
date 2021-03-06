#![warn(unused_extern_crates)]
use bip39::Mnemonic;
use clap::{crate_version, Parser};
mod commands;
mod lib;

/// Ledger & Governance ToolKit for cold wallets.
#[derive(Parser)]
#[clap(name("quill"), version = crate_version!())]
pub struct CliOpts {
    /// Path to your seed file (use "-" for STDIN)
    #[clap(long)]
    seed_file: Option<String>,

    /// Path to your PEM file (use "-" for STDIN)
    #[clap(long)]
    pem_file: Option<String>,

    #[clap(subcommand)]
    command: commands::Command,
}

fn main() {
    let opts = CliOpts::parse();
    let command = opts.command;
    let pem = match opts.pem_file {
        Some(file) => Some(read_file(file)),
        None => opts.seed_file.map(|path| {
            let phrase = read_file(path);
            lib::mnemonic_to_pem(
                &Mnemonic::parse(phrase)
                    .expect("Couldn't parse the seed phrase as a valid mnemonic"),
            )
        }),
    };
    if let Err(err) = commands::exec(pem, command) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn read_file(path: String) -> String {
    match path.as_str() {
        // read from STDIN
        "-" => {
            let mut buffer = String::new();
            use std::io::Read;
            if let Err(err) = std::io::stdin().read_to_string(&mut buffer) {
                eprintln!("Couldn't read from STDIN: {:?}", err);
                std::process::exit(1);
            }
            buffer
        }
        path => std::fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Couldn't read file: {:?}", err);
            std::process::exit(1);
        }),
    }
}
