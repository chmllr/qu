use crate::lib::{
    get_idl_string, read_from_file, request_status, send_ingress,
    signing::{Ingress, IngressWithRequestId},
    AnyhowResult, IngressResult,
};
use anyhow::anyhow;
use clap::Parser;

/// Sends a signed message or a set of messages.
#[derive(Parser)]
pub struct Opts {
    /// Path to the signed message
    file_name: String,

    /// Will display the signed message, but not send it.
    #[clap(long)]
    dry_run: bool,

    /// Skips confirmation and sends the message directly.
    #[clap(long)]
    yes: bool,

    /// Print raw output
    #[clap(long)]
    raw: bool,
}

pub async fn exec(opts: Opts) -> AnyhowResult {
    let json = read_from_file(&opts.file_name)?;
    if let Ok(val) = serde_json::from_str::<Ingress>(&json) {
        send(&val, &opts).await?;
    } else if let Ok(vals) = serde_json::from_str::<Vec<Ingress>>(&json) {
        for msg in vals {
            send(&msg, &opts).await?;
        }
    } else if let Ok(vals) = serde_json::from_str::<Vec<IngressWithRequestId>>(&json) {
        for tx in vals {
            submit_ingress_and_check_status(&tx, &opts).await?;
        }
    } else {
        return Err(anyhow!("Invalid JSON content"));
    }
    Ok(())
}

async fn submit_ingress_and_check_status(
    message: &IngressWithRequestId,
    opts: &Opts,
) -> AnyhowResult {
    send(&message.ingress, opts).await?;
    if opts.dry_run {
        return Ok(());
    }
    let (_, canister_id, method_name, _) = &message.ingress.parse()?;
    let silent = opts.raw;
    match request_status::submit(&message.request_status, silent).await {
        Ok(blob) if opts.raw => {
            use std::io::Write;
            let mut out = std::io::stdout();
            out.write_all(&blob)?;
            out.flush()?;
        }
        Ok(blob) => {
            let response = crate::lib::get_idl_string(&blob, *canister_id, &method_name, "rets");
            println!("{}\n", response.map_err(|e| anyhow!(e))?)
        }
        Err(err) => println!("{}\n", err),
    };
    Ok(())
}

async fn send(message: &Ingress, opts: &Opts) -> AnyhowResult {
    let (sender, canister_id, method_name, args) = message.parse()?;

    if !opts.raw {
        println!("Sending message with\n");
        println!("  Call type:   {}", message.call_type);
        println!("  Sender:      {}", sender);
        println!("  Canister id: {}", canister_id);
        println!("  Method name: {}", method_name);
        println!("  Arguments:   {}", args.map_err(|e| anyhow!(e))?);
    }

    if opts.dry_run {
        return Ok(());
    }

    if message.call_type == "update" && !opts.yes {
        println!("\nDo you want to send this message? [y/N]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !["y", "yes"].contains(&input.to_lowercase().trim()) {
            std::process::exit(0);
        }
    }

    match send_ingress(message).await? {
        IngressResult::QueryResponse(response) => {
            if opts.raw {
                write_to_stdout(&response)?;
            } else {
                println!(
                    "Response: {}",
                    get_idl_string(&response, canister_id, &method_name, "rets")
                        .map_err(|e| anyhow!(e))?
                );
            }
        }
        IngressResult::RequestId(id) => {
            if !opts.raw {
                println!("RequestId: 0x{}", String::from(id));
            }
        }
    };
    Ok(())
}

fn write_to_stdout(blob: &[u8]) -> AnyhowResult {
    use std::io::Write;
    let mut out = std::io::stdout();
    out.write_all(blob)?;
    out.flush()?;
    Ok(())
}
