use crate::lib::{
    governance_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult,
};
use anyhow::anyhow;
use candid::Encode;
use clap::Parser;
use ic_agent::Agent;
use ic_base_types::PrincipalId;
use ic_nns_common::pb::v1::NeuronId;
use ic_nns_governance::pb::v1::{
    manage_neuron::{
        configure::Operation, AddHotKey, Command, Configure, Disburse, IncreaseDissolveDelay,
        Merge, MergeMaturity, RemoveHotKey, Split, StartDissolving, StopDissolving,
    },
    ManageNeuron,
};

/// Signs a neuron configuration change.
#[derive(Parser)]
pub struct Opts {
    /// The id of the neuron to manage.
    neuron_id: String,

    /// Principal to be used as a hot key.
    #[clap(long)]
    add_hot_key: Option<PrincipalId>,

    /// Principal hot key to be removed.
    #[clap(long)]
    remove_hot_key: Option<PrincipalId>,

    /// Number of dissolve seconds to add.
    #[clap(short, long)]
    additional_dissolve_delay_seconds: Option<String>,

    /// Start dissolving.
    #[clap(long)]
    start_dissolving: bool,

    /// Stop dissolving.
    #[clap(long)]
    stop_dissolving: bool,

    /// Disburse the entire staked amount to the controller's account.
    #[clap(long)]
    disburse: bool,

    /// Spawn rewards to a new neuron under the controller's account.
    #[clap(long)]
    spawn: bool,

    /// Split off the given number of ICP from a neuron.
    #[clap(long)]
    split: Option<u64>,

    /// Merge stake, maturity and age from the neuron specified by this option into the neuron being managed.
    #[clap(long)]
    merge_from_neuron: Option<String>,

    /// Merge the percentage (between 1 and 100) of the maturity of a neuron into the current stake.
    #[clap(long)]
    merge_maturity: Option<u32>,
}

pub fn exec(agent: Agent, opts: Opts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let mut msgs = Vec::new();

    let id = Some(NeuronId {
        id: parse_neuron_id(opts.neuron_id),
    });
    if opts.add_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::AddHotKey(AddHotKey {
                    new_hot_key: opts.add_hot_key
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.remove_hot_key.is_some() {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::RemoveHotKey(RemoveHotKey {
                    hot_key_to_remove: opts.remove_hot_key
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.stop_dissolving {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StopDissolving(StopDissolving {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if opts.start_dissolving {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::StartDissolving(StartDissolving {}))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    }

    if let Some(additional_dissolve_delay_seconds) = opts.additional_dissolve_delay_seconds {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Configure(Configure {
                operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds: additional_dissolve_delay_seconds
                        .parse::<u32>()
                        .expect("Couldn't parse the dissolve delay"),
                }))
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.disburse {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Disburse(Disburse {
                to_account: None,
                amount: None
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if opts.spawn {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Spawn(Default::default())),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if let Some(amount) = opts.split {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Split(Split {
                amount_e8s: amount * 100_000_000
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if let Some(neuron_id) = opts.merge_from_neuron {
        let args = Encode!(&ManageNeuron {
            id: id.clone(),
            command: Some(Command::Merge(Merge {
                source_neuron_id: Some(NeuronId {
                    id: parse_neuron_id(neuron_id)
                }),
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if let Some(percentage_to_merge) = opts.merge_maturity {
        if percentage_to_merge == 0 || percentage_to_merge > 100 {
            return Err(anyhow!(
                "Percentage to merge must be a number from 1 to 100"
            ));
        }
        let args = Encode!(&ManageNeuron {
            id,
            command: Some(Command::MergeMaturity(MergeMaturity {
                percentage_to_merge
            })),
            neuron_id_or_subaccount: None,
        })?;
        msgs.push(args);
    };

    if msgs.is_empty() {
        return Err(anyhow!("No instructions provided"));
    }

    let mut generated = Vec::new();
    for args in msgs {
        generated.push(sign_ingress_with_request_status_query(
            agent.clone(),
            governance_canister_id(),
            "manage_neuron",
            args,
        )?);
    }
    Ok(generated)
}

fn parse_neuron_id(id: String) -> u64 {
    id.replace('_', "")
        .parse()
        .expect("Couldn't parse the neuron id")
}
