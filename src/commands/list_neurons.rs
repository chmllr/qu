use crate::{
    lib::signing::{sign_ingress, Ingress},
    lib::{governance_canister_id, AnyhowResult},
};
use candid::Encode;
use clap::Parser;
use ic_agent::Agent;
use ic_nns_governance::pb::v1::ListNeurons;

/// Signs a neuron configuration change.
#[derive(Parser)]
pub struct Opts {
    /// The optional ids of the specific neuron to query. Note that these ids
    /// may only be those that occur in the usual output from `list-neurons`,
    /// i.e., they should be ids of the user's own neurons. The purpose of
    /// this option is to narrow the query, and not to allow querying of
    /// arbtirary neuron ids.
    neuron_id: Vec<u64>,
}

// We currently only support a subset of the functionality.
pub fn exec(agent: Agent, opts: Opts) -> AnyhowResult<Vec<Ingress>> {
    let args = Encode!(&ListNeurons {
        neuron_ids: opts.neuron_id.clone(),
        include_neurons_readable_by_caller: opts.neuron_id.is_empty(),
    })?;
    let method_name = "list_neurons";
    let canister_id = governance_canister_id();
    Ok(vec![sign_ingress(
        agent,
        canister_id,
        method_name,
        crate::lib::is_query(canister_id, method_name),
        args,
    )?])
}
