//! This module implements the command-line API.

use crate::{
    commands::raw::IngressMessage,
    lib::{require_pem, AnyhowResult},
};
use clap::Parser;
use std::io::{self, Write};
use tokio::runtime::Runtime;

mod generate;
mod ids;
mod list_neurons;
mod neuron_manage;
mod neuron_stake;
mod raw;
mod send;
mod transfer;

pub use ids::get_ids;

#[derive(Parser)]
pub enum Command {
    /// Prints the principal id and the account id.
    PublicIds,
    Send(send::Opts),
    Transfer(transfer::Opts),
    NeuronStake(neuron_stake::Opts),
    NeuronManage(neuron_manage::Opts),
    /// Signs the query for all neurons belonging to the signing principal.
    ListNeurons(list_neurons::Opts),
    /// Generate a mnemonic seed phrase and generate or recover PEM.
    Generate(generate::Opts),
    /// Call a canister method directly
    Raw(raw::Opts),
}

pub fn exec(pem: &Option<String>, cmd: Command) -> AnyhowResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    match cmd {
        Command::PublicIds => ids::exec(pem),
        Command::Transfer(opts) => {
            let pem = require_pem(pem)?;
            transfer::exec(&pem, opts).and_then(|out| print(&out))
        }
        Command::NeuronStake(opts) => {
            let pem = require_pem(pem)?;
            neuron_stake::exec(&pem, opts).and_then(|out| print(&out))
        }
        Command::NeuronManage(opts) => {
            let pem = require_pem(pem)?;
            neuron_manage::exec(&pem, opts).and_then(|out| print(&out))
        }
        Command::ListNeurons(opts) => {
            let pem = require_pem(pem)?;
            list_neurons::exec(&pem, opts).and_then(|out| print(&out))
        }
        Command::Send(opts) => runtime.block_on(async { send::exec(opts).await }),
        Command::Generate(opts) => generate::exec(opts),
        Command::Raw(opts) => {
            let pem = require_pem(pem)?;
            raw::exec(&pem, opts).and_then(|out| match out {
                IngressMessage::Ingress(msg) => print(&vec![msg]),
                IngressMessage::IngressWithRequestId(msg) => print(&vec![msg]),
            })
        }
    }
}

// Using println! for printing to STDOUT and piping it to other tools leads to
// the problem that when the other tool closes its stream, the println! macro
// panics on the error and the whole binary crashes. This function provides a
// graceful handling of the error.
fn print<T>(arg: &T) -> AnyhowResult
where
    T: ?Sized + serde::ser::Serialize,
{
    if let Err(e) = io::stdout().write_all(serde_json::to_string(&arg)?.as_bytes()) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
