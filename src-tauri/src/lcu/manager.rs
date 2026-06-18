use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{
    app::AppContext,
    core::events::AppEvent,
    lcu::{http::LcuHttpClient, ws},
    models::MonitorLevel,
};

pub async fn run_connection_manager(context: Arc<AppContext>, shutdown: CancellationToken) {
    let mut receiver = context.bus.subscribe();
    let mut ws_token: Option<CancellationToken> = None;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                if let Some(token) = ws_token.take() {
                    token.cancel();
                }
                break;
            }
            event_result = receiver.recv() => {
                let Ok(event) = event_result else {
                    continue;
                };

                match event {
                    AppEvent::LockfileDetected(lockfile) => {
                        match LcuHttpClient::from_lockfile(&lockfile) {
                            Ok(client) => {
                                context.set_lcu_client(Some(client)).await;
                                context.bus.publish(AppEvent::ClientConnected { port: lockfile.port });
                                context.monitor(
                                    MonitorLevel::Info,
                                    "lcu",
                                    format!(
                                        "League client detected (pid {}, port {}, protocol {}, process {})",
                                        lockfile.pid, lockfile.port, lockfile.protocol, lockfile.process_name
                                    ),
                                );
                                context.monitor(
                                    MonitorLevel::Info,
                                    "lcu",
                                    format!("Watching lockfile at {}", lockfile.lockfile_path.display()),
                                );

                                if let Some(previous) = ws_token.take() {
                                    previous.cancel();
                                }

                                let child_token = shutdown.child_token();
                                ws_token = Some(child_token.clone());
                                let bus = context.bus.clone();
                                tauri::async_runtime::spawn(async move {
                                    ws::run_ws_loop(bus, lockfile, child_token).await;
                                });
                            }
                            Err(error) => {
                                context.monitor(
                                    MonitorLevel::Error,
                                    "lcu",
                                    format!("Failed to initialize LCU client: {}", error),
                                );
                            }
                        }
                    }
                    AppEvent::LockfileMissing => {
                        context.set_lcu_client(None).await;
                        if let Some(token) = ws_token.take() {
                            token.cancel();
                        }
                        context.bus.publish(AppEvent::ClientDisconnected);
                        context.monitor(
                            MonitorLevel::Warn,
                            "lcu",
                            "League client lockfile not found. Waiting for restart.".to_string(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

