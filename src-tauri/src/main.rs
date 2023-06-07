// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use rinna_gui::util::short_sleep;
use rinna_gui::{exit_rinna, init_rinna, ExecRinnaRet};
use std::sync::Mutex;
use tauri::Manager;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

#[derive(serde::Deserialize, Debug)]
struct Request {
    id: usize,
    prompt: String,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(tag = "type")]
enum Response {
    Reply { id: usize, response: String },
    End { id: usize },
}

fn prepare_request_handler_thread(
    prompt_tx: Sender<String>,
    mut token_rx: Receiver<()>,
    mut rinna_response_rx: Receiver<Option<String>>,
    standby_tx: Sender<()>,
) -> (Sender<Request>, Receiver<Response>, JoinHandle<Result<()>>) {
    let (request_tx, mut request_rx) = channel(256);
    let (response_tx, response_rx) = channel(256);

    let thread: JoinHandle<Result<()>> = tokio::spawn(async move {
        while let Some(()) = token_rx.recv().await {
            standby_tx
                .send(())
                .await
                .context("Failed to send standby")?;
            let Some(Request { id, prompt }) = request_rx.recv().await else {
                break;
            };

            prompt_tx
                .send(prompt)
                .await
                .context("Failed to send prompt")?;

            while let Some(r) = rinna_response_rx.recv().await {
                match r {
                    Some(response) => {
                        for c in response.clone().chars() {
                            response_tx
                                .send(Response::Reply {
                                    id,
                                    response: c.to_string(),
                                })
                                .await
                                .context("Failed to send response")?;
                        }
                    }
                    None => {
                        response_tx
                            .send(Response::End { id })
                            .await
                            .context("Failed to send response")?;
                        break;
                    }
                }
            }
        }

        Ok(())
    });

    (request_tx, response_rx, thread)
}

#[tauri::command]
async fn rinna(
    id: usize,
    prompt: String,
    request_tx: tauri::State<'_, Sender<Request>>,
) -> Result<(), String> {
    let prompt = prompt.replace("[exit]", "#exit#");
    short_sleep(100).await;

    request_tx
        .send(Request { id, prompt })
        .await
        .map_err(|e| format!("Error!: {:?}", e))?;

    Ok(())
}

struct ReactReady(bool);

#[tauri::command]
async fn react_ready(ready_state: tauri::State<'_, Mutex<ReactReady>>) -> Result<(), String> {
    let mut ready_state = ready_state.lock().unwrap();
    ready_state.0 = true;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    env_logger::init();

    log::info!("Logging Start");

    let ExecRinnaRet {
        child,
        prompt_tx,
        token_rx,
        response_rx: rinna_response_rx,
        output_handle,
        input_handle,
    } = init_rinna("./tmp.exe").await?;

    let pt = prompt_tx.clone();
    let (exit_tx, mut exit_rx) = channel(1);
    let exit_handle = tokio::spawn(async move {
        if let Some(_) = exit_rx.recv().await {
            if let Err(e) = exit_rinna(child, pt).await {
                log::error!("@ Error while exit_rinna: {:?}", e);
            }
        }
    });

    let (standby_tx, mut standby_rx) = channel(1);
    let (request_tx, mut response_rx, session_handle) =
        prepare_request_handler_thread(prompt_tx, token_rx, rinna_response_rx, standby_tx);

    let ready_state = Mutex::new(ReactReady(false));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rinna, react_ready])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();

            // standby thread
            let mw = main_window.clone();
            tokio::spawn(async move {
                while let false = mw.state::<Mutex<ReactReady>>().lock().unwrap().0 {}
                short_sleep(100).await;

                while let Some(_) = standby_rx.recv().await {
                    log::info!("Standby");
                    let r = mw.emit("rinna-standby", ());

                    if let Err(e) = r {
                        log::warn!("Warn! @ Error while emit: {:?}", e);
                    }
                }
            });

            // response thread
            let mw = main_window.clone();
            tokio::spawn(async move {
                while let false = mw.state::<Mutex<ReactReady>>().lock().unwrap().0 {}
                short_sleep(100).await;

                while let Some(res) = response_rx.recv().await {
                    log::info!("Response: {:?}", res);
                    short_sleep(10).await;
                    let r = mw.emit("rinna-response", res);

                    if let Err(e) = r {
                        log::warn!("Warn! @ Error while emit: {:?}", e);
                    }
                }
            });

            Ok(())
        })
        .on_window_event(move |event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                let etx = exit_tx.clone();
                tokio::spawn(async move {
                    etx.send(()).await.unwrap();
                });
            }
            _ => {}
        })
        .manage(request_tx)
        .manage(ready_state)
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    match tokio::try_join!(output_handle, input_handle, session_handle, exit_handle) {
        Ok((first, second, third, _forth)) => {
            first.context("Failed to join output_handle")?;
            second.context("Failed to join input_handle")?;
            third.context("Failed to join request_handle")?;
        }
        Err(e) => {
            log::error!("@ Error while Join: {:?}", e);
        }
    }

    Ok(())
}
