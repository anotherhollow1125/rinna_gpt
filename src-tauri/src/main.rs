// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use rinna_gui::util::short_sleep;
use rinna_gui::{exit_rinna, init_rinna, ExecRinnaRet};
use std::env;
use std::sync::mpsc::channel as std_channel;
use std::sync::Mutex;
use tauri::{App, Manager};
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

            // Clear the response queue
            while let Ok(_) = rinna_response_rx.try_recv() {}

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

use std::path::{Path, PathBuf};

fn rinna_path(app: &mut App) -> PathBuf {
    /* // for debug
    let mut rinna_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    rinna_path.push("rinna.exe");
    */

    let rinna_path = if let Ok(p) = env::var("RINNA_EXE") {
        PathBuf::from(p)
    } else {
        let d = app
            .path_resolver()
            .resolve_resource("rinna.exe")
            .expect("failed to resolve resource");
        log::info!("Default rinna.exe path: {:?}", d);
        d
    };

    rinna_path
}

struct SetupRes {
    standby_rx: Receiver<()>,
    response_rx: Receiver<Response>,
    exit_tx: Sender<()>,
    request_tx: Sender<Request>,
    output_handle: JoinHandle<Result<()>>,
    input_handle: JoinHandle<Result<()>>,
    session_handle: JoinHandle<Result<()>>,
}

async fn rinna_setup(exe_path: impl AsRef<Path>) -> Result<SetupRes> {
    let ExecRinnaRet {
        child,
        prompt_tx,
        token_rx,
        response_rx: rinna_response_rx,
        output_handle,
        input_handle,
    } = init_rinna(&exe_path).await?;

    let pt = prompt_tx.clone();
    let (exit_tx, mut exit_rx) = channel::<()>(1);
    let _exit_handle = tokio::spawn(async move {
        if let Some(_) = exit_rx.recv().await {
            if let Err(e) = exit_rinna(child, pt).await {
                log::error!("@ Error while exit_rinna: {:?}", e);
            }
        }
    });

    let (standby_tx, standby_rx) = channel(1);
    let (request_tx, response_rx, session_handle) =
        prepare_request_handler_thread(prompt_tx, token_rx, rinna_response_rx, standby_tx);

    Ok(SetupRes {
        standby_rx,
        response_rx,
        exit_tx,
        request_tx,
        output_handle,
        input_handle,
        session_handle,
    })
}

#[tauri::command]
async fn rinna(
    id: usize,
    prompt: String,
    request_tx: tauri::State<'_, Sender<Request>>,
    rinna_standby: tauri::State<'_, Mutex<RinnaStandby>>,
) -> Result<(), String> {
    {
        let mut standby_state = rinna_standby.lock().unwrap();
        standby_state.0 = false;
    }

    let prompt = prompt.replace("[exit]", "#exit#");
    short_sleep(100).await;

    request_tx
        .send(Request { id, prompt })
        .await
        .map_err(|e| format!("Error!: {:?}", e))?;

    Ok(())
}

struct ReactReady(bool);
struct RinnaStandby(bool);

#[tauri::command]
async fn react_ready(ready_state: tauri::State<'_, Mutex<ReactReady>>) -> Result<(), String> {
    let mut ready_state = ready_state.lock().unwrap();
    ready_state.0 = true;
    Ok(())
}

#[tauri::command]
async fn is_rinna_standby(
    standby_state: tauri::State<'_, Mutex<RinnaStandby>>,
) -> Result<bool, String> {
    let standby_state = standby_state.lock().unwrap();
    Ok(standby_state.0)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    env_logger::init();

    log::info!("Logging Start");

    let react_ready_state = Mutex::new(ReactReady(false));
    let rinna_standby_state = Mutex::new(RinnaStandby(false));

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            rinna,
            react_ready,
            is_rinna_standby
        ])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();

            let rinna_exe_path = rinna_path(app);

            let (setup_tx, setup_rx) = std_channel();
            tokio::spawn(async move {
                let setup_res = rinna_setup(&rinna_exe_path).await;
                if let Err(e) = setup_res {
                    log::error!("Error while rinna_setup: {:?}", e);
                } else {
                    setup_tx.send(setup_res.unwrap()).unwrap();
                }
            });

            let SetupRes {
                mut standby_rx,
                mut response_rx,
                exit_tx,
                request_tx,
                output_handle,
                input_handle,
                session_handle,
            } = setup_rx.recv().unwrap();

            // standby thread
            let mw = main_window.clone();
            tokio::spawn(async move {
                while let false = mw.state::<Mutex<ReactReady>>().lock().unwrap().0 {}
                short_sleep(100).await;

                while let Some(_) = standby_rx.recv().await {
                    log::info!("Standby");
                    {
                        let s = mw.state::<Mutex<RinnaStandby>>();
                        let mut standby_state = s.lock().unwrap();
                        standby_state.0 = true;
                    }
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

            main_window.on_window_event(move |event| match event {
                tauri::WindowEvent::CloseRequested { .. } => {
                    let etx = exit_tx.clone();
                    tokio::spawn(async move {
                        etx.send(()).await.unwrap();
                    });
                }
                _ => {}
            });
            app.manage(request_tx);

            // check Results of JoinHandle<Result<()>>s
            tokio::spawn(async move {
                match tokio::try_join!(output_handle, input_handle, session_handle) {
                    Ok((first, second, third)) => {
                        if let Err(e) = first {
                            log::error!("@ Error while Join: {:?}", e);
                        }
                        if let Err(e) = second {
                            log::error!("@ Error while Join: {:?}", e);
                        }
                        if let Err(e) = third {
                            log::error!("@ Error while Join: {:?}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("@ Error while Join: {:?}", e);
                    }
                }
            });

            Ok(())
        })
        .manage(react_ready_state)
        .manage(rinna_standby_state)
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}
