use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult,
};
use anyhow::anyhow;
use candid::Principal;
use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr;

/// Raw canister call
#[derive(Parser)]
pub struct Opts {
    /// Canister id
    canister_id: Principal,

    /// Canister method
    method: String,

    /// Method arguments as a Candid string
    args: Option<String>,

    /// Binary file
    args_file: Option<PathBuf>,
}

pub fn exec(pem: &str, opts: Opts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let bytes = match (&opts.args, &opts.args_file) {
        (Some(args), None) => candid::IDLArgs::from_str(&args)?.to_bytes()?,
        (None, Some(path)) => {
            use std::{
                fs::File,
                io::{BufReader, Read},
            };
            let mut reader = BufReader::new(File::open(path)?);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;
            buffer
        }
        _ => {
            return Err(anyhow!(
                "String args or a file with argument bytes should be specified".to_owned(),
            ))
        }
    };
    Ok(vec![sign_ingress_with_request_status_query(
        pem,
        opts.canister_id,
        &opts.method,
        bytes,
    )?])
}
