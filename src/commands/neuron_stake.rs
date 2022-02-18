use crate::{
    commands::transfer,
    lib::{
        governance_canister_id,
        signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
        AnyhowResult,
    },
};
use anyhow::anyhow;
use candid::Encode;
use clap::Parser;
use ic_base_types::PrincipalId;
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_nns_governance::{
    governance::compute_neuron_staking_subaccount, pb::v1::ClaimOrRefreshNeuronFromAccount,
};
use ledger_canister::AccountIdentifier;
use std::str::FromStr;

/// Signs topping up of a neuron (new or existing).
#[derive(Parser)]
pub struct Opts {
    /// ICPs to be staked on the newly created neuron.
    #[clap(long)]
    amount: Option<String>,

    /// The name of the neuron (up to 8 ASCII characters).
    #[clap(long, validator(neuron_name_validator))]
    name: Option<String>,

    /// The nonce of the neuron.
    #[clap(long, validator(neuron_name_validator), conflicts_with("name"))]
    nonce: Option<u64>,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long)]
    fee: Option<String>,
}

pub fn exec(pem: &str, opts: Opts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let (controller, _) = crate::commands::ids::get_ids(&Some(pem.to_string()))?;
    let nonce = match (&opts.nonce, &opts.name) {
        (Some(nonce), _) => *nonce,
        (_, Some(name)) => convert_name_to_nonce(name),
        _ => return Err(anyhow!("Either a nonce or a name should be specified")),
    };
    let subacc = compute_neuron_staking_subaccount(
        PrincipalId::from_str(&controller.to_string()).expect("couldn't parse principal id"),
        nonce,
    );
    let mut messages = match opts.amount {
        Some(amount) => transfer::exec(
            pem,
            transfer::Opts {
                to: AccountIdentifier::new(GOVERNANCE_CANISTER_ID.get(), Some(subacc)),
                amount,
                fee: opts.fee,
                memo: Some(nonce.to_string()),
            },
        )?,
        _ => Vec::new(),
    };
    let args = Encode!(&ClaimOrRefreshNeuronFromAccount {
        memo: nonce,
        controller: Some(controller),
    })?;

    messages.push(sign_ingress_with_request_status_query(
        pem,
        governance_canister_id(),
        "claim_or_refresh_neuron_from_account",
        args,
    )?);

    Ok(messages)
}

fn convert_name_to_nonce(name: &str) -> u64 {
    let mut bytes = std::collections::VecDeque::from(name.as_bytes().to_vec());
    while bytes.len() < 8 {
        bytes.push_front(0)
    }
    let mut arr: [u8; 8] = [0; 8];
    arr.copy_from_slice(&bytes.into_iter().collect::<Vec<_>>());
    u64::from_be_bytes(arr)
}

fn neuron_name_validator(name: &str) -> Result<(), String> {
    // Convert to bytes before checking the length to restrict it to ASCII only
    if name.as_bytes().len() > 8 {
        return Err("The neuron name must be 8 character or less".to_string());
    }
    Ok(())
}
