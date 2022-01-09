use crate::lib::{get_account_id, get_identity, require_pem, AnyhowResult};
use anyhow::anyhow;
use ic_types::principal::Principal;
use ledger_canister::AccountIdentifier;

/// Prints the account and the principal ids.
pub fn exec(pem: &Option<String>) -> AnyhowResult {
    let (principal_id, account_id) = get_ids(pem)?;
    println!("Principal id: {}", principal_id.to_text());
    println!("Account id: {}", account_id);
    Ok(())
}

/// Returns the account id and the principal id if the private key was provided.
pub fn get_ids(pem: &Option<String>) -> AnyhowResult<(Principal, AccountIdentifier)> {
    require_pem(pem)?;
    let principal_id = get_identity(pem.as_ref().unwrap())
        .sender()
        .map_err(|e| anyhow!(e))?;
    Ok((principal_id, get_account_id(principal_id)?))
}
