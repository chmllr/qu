use std::str::FromStr;

use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult,
};
use clap::Parser;

/// Raw governance canister call
#[derive(Parser)]
pub struct Opts {
    /// Canister method
    method: String,

    /// Method arguments as a Candid string
    args: String,
}

pub fn exec(pem: &str, opts: Opts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    Ok(vec![sign_ingress_with_request_status_query(
        pem,
        governance_canister_id(),
        &opts.method,
        candid::IDLArgs::from_str(&opts.args)?.to_bytes()?,
    )?])
}
