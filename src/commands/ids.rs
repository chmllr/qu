use crate::lib::{get_account_id, AnyhowResult};
use ic_agent::Agent;
use ic_base_types::PrincipalId;
use ledger_canister::AccountIdentifier;

/// Prints the account and the principal ids.
pub fn exec(agent: Agent) -> AnyhowResult {
    let (principal_id, account_id) = get_ids(&agent)?;
    println!("Principal id: {}", principal_id);
    println!("Account id: {}", account_id);
    Ok(())
}

/// Returns the account id and the principal id if the private key was provided.
pub fn get_ids(agent: &Agent) -> AnyhowResult<(PrincipalId, AccountIdentifier)> {
    let principal_id = agent.get_principal().expect("couldn't get principal");
    Ok((PrincipalId(principal_id), get_account_id(principal_id)?))
}
