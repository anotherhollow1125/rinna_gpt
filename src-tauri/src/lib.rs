pub mod util;

use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

pub struct ExecRinnaRet {
    pub child: Child,
    pub prompt_tx: Sender<String>,
    pub token_rx: Receiver<()>,
    pub response_rx: Receiver<Option<String>>,
    pub output_handle: JoinHandle<Result<()>>,
    pub input_handle: JoinHandle<Result<()>>,
}

pub async fn init_rinna(rinna_path: impl AsRef<Path>) -> Result<ExecRinnaRet> {
    let mut child = Command::new(rinna_path.as_ref())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn rinna.exe")?;

    let mut stdin = child.stdin.take().context("Failed to open stdin")?;
    let mut stdout_ = child.stdout.take().context("Failed to open stdout")?;

    let (prompt_tx, mut prompt_rx) = channel::<String>(1);
    let (token_tx, token_rx) = channel::<()>(1);
    let (response_tx, response_rx) = channel::<Option<String>>(1);

    let input_handle = tokio::spawn(async move {
        while let Some(prompt) = prompt_rx.recv().await {
            let prompt = format!("{}\n", prompt.replace("\n", "<NL>"));
            let prompt = CString::new(prompt)?;
            stdin
                .write_all(prompt.as_bytes())
                .await
                .context("Failed to write to stdin")?;
        }

        Ok(())
    });

    let output_handle = tokio::spawn(async move {
        let mut query_counter = 0;

        loop {
            let mut temp_buf = [0; 256];
            let word = match stdout_.read(&mut temp_buf).await? {
                0 => break,
                n => String::from_utf8_lossy(&temp_buf[..n]).to_string(),
            };

            log::info!("Rinna: {}", word);

            let response = word.replace("> ", "").to_string();

            if response.len() > 0 {
                response_tx
                    .send(Some(response))
                    .await
                    .context("Failed to send response")?;
            }

            if word.ends_with("> ") {
                if query_counter > 0 {
                    response_tx
                        .send(None)
                        .await
                        .context("Failed to send response")?;
                }
                token_tx.send(()).await.context("Failed to send token")?;
                query_counter += 1;

                log::info!("Rinna is ready");
            }
        }

        Ok(())
    });

    Ok(ExecRinnaRet {
        child,
        prompt_tx,
        token_rx,
        response_rx,
        output_handle,
        input_handle,
    })
}

pub async fn exit_rinna(mut rinna: Child, prompt_tx: Sender<String>) -> Result<()> {
    prompt_tx
        .send("[exit]".to_string())
        .await
        .context("Failed to send [exit]")?;

    let status = rinna.wait().await.context("Failed to wait for rinna.exe")?;

    if !status.success() {
        return Err(anyhow!("!! rinna.exe exited with status: {}", status));
    }

    Ok(())
}
