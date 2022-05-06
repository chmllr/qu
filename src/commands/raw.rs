use std::str::FromStr;

use crate::lib::{
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult,
};
use candid::Principal;
use clap::Parser;

/// Raw canister call
#[derive(Parser)]
pub struct Opts {
    /// Canister id
    canister_id: Principal,

    /// Canister method
    method: String,

    /// Method arguments as a Candid string
    args: String,
}

pub fn exec(pem: &str, opts: Opts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    Ok(vec![sign_ingress_with_request_status_query(
        pem,
        opts.canister_id,
        &opts.method,
        candid::IDLArgs::from_str(&opts.args)?.to_bytes()?,
    )?])
}
