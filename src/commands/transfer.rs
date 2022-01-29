use crate::lib::{
    ledger_canister_id,
    signing::{sign_ingress_with_request_status_query, IngressWithRequestId},
    AnyhowResult,
};
use anyhow::anyhow;
use candid::Encode;
use clap::Parser;
use ledger_canister::{AccountIdentifier, Memo, SendArgs, Tokens, TRANSACTION_FEE};

/// Signs an ICP transfer transaction.
#[derive(Parser)]
pub struct TransferOpts {
    /// Reference number, default is 0.
    #[clap(long)]
    pub memo: Option<String>,

    /// Amount of ICPs to transfer (with up to 8 decimal digits after comma).
    #[clap(long)]
    pub amount: String,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long)]
    pub fee: Option<String>,

    /// Destination account.
    pub to: AccountIdentifier,
}

pub fn exec(pem: &str, opts: TransferOpts) -> AnyhowResult<Vec<IngressWithRequestId>> {
    let amount =
        parse_icpts(&opts.amount).map_err(|err| anyhow!("Could not add ICPs and e8s: {}", err))?;
    let fee = opts.fee.map_or(TRANSACTION_FEE, |v| {
        parse_icpts(&v).expect("couldn't parse fees")
    });
    let memo = Memo(
        opts.memo
            .unwrap_or_else(|| "0".to_string())
            .parse::<u64>()
            .unwrap(),
    );
    let to = opts.to;

    let args = Encode!(&SendArgs {
        memo,
        amount,
        fee,
        from_subaccount: None,
        to,
        created_at_time: None,
    })?;

    let msg = sign_ingress_with_request_status_query(pem, ledger_canister_id(), "send_dfx", args)?;
    Ok(vec![msg])
}

fn parse_icpts(amount: &str) -> Result<Tokens, String> {
    let parse = |s: &str| {
        s.parse::<u64>()
            .map_err(|err| format!("Couldn't parse as u64: {:?}", err))
    };
    match &amount.split('.').collect::<Vec<_>>().as_slice() {
        [icpts] => Tokens::new(parse(icpts)?, 0),
        [icpts, e8s] => {
            let mut e8s = e8s.to_string();
            while e8s.len() < 8 {
                e8s.push('0');
            }
            let e8s = &e8s[..8];
            Tokens::new(parse(icpts)?, parse(e8s)?)
        }
        _ => Err(format!("Can't parse amount {}", amount)),
    }
}
