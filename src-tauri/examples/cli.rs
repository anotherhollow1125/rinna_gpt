use anyhow::{anyhow, Context, Result};
use rinna_gui::{exit_rinna, init_rinna, ExecRinnaRet};
use std::io::{stdout, Write};
use std::path::Path;
use tokio::task::JoinHandle;

fn print_without_ln(s: &str) -> Result<()> {
    print!("{}", s);
    stdout().flush().context("Failed to flush stdout")?;

    Ok(())
}

use dialoguer::Input;

#[tokio::main]
async fn main() -> Result<()> {
    // println!("{}", std::env::current_dir().unwrap().display());
    println!("Please wait for a while...");

    let true = Path::new("./rinna.exe").exists() else {
        return Err(anyhow!("!! rinna.exe does not exist"));
    };

    let ExecRinnaRet {
        child,
        prompt_tx,
        mut token_rx,
        mut response_rx,
        output_handle,
        input_handle,
    } = init_rinna("./rinna.exe").await?;

    let print_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        while let Some(res) = response_rx.recv().await {
            if let Some(response) = res {
                print_without_ln(&response)?;
            } else {
                println!("==========");
            }
        }

        Ok(())
    });

    while let Some(()) = token_rx.recv().await {
        let prompt = Input::<String>::new()
            .with_prompt("User")
            .interact()
            .unwrap();

        print_without_ln("Rinna: ")?;

        if prompt == "[exit]" {
            exit_rinna(child, prompt_tx).await?;
            break;
        }

        prompt_tx
            .send(prompt)
            .await
            .context("Failed to send prompt")?;
    }

    match tokio::try_join!(output_handle, input_handle, print_handle) {
        Ok((first, second, third)) => {
            first.context("Failed to join output_handle")?;
            second.context("Failed to join input_handle")?;
            third.context("Failed to join print_handle")?;
        }
        Err(e) => {
            eprintln!("@ Error while Join: {:?}", e);
        }
    }

    Ok(())
}
