use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_dialog::{
    DialogExt, MessageDialogBuilder, MessageDialogButtons, MessageDialogKind,
};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_shell::{process::CommandChild, ShellExt};

const GPTME_SERVER_PORT: u16 = 5700;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Check if a port is available
fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("gptme-tauri".to_string()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            log::info!("Starting gptme-tauri application");

            let app_handle = app.handle().clone();

            // Store child process reference for cleanup
            let child_process: Arc<Mutex<Option<CommandChild>>> = Arc::new(Mutex::new(None));
            let child_for_cleanup = child_process.clone();

            // Spawn gptme-server with output capture
            tauri::async_runtime::spawn(async move {
                // Check if port is available before starting
                if !is_port_available(GPTME_SERVER_PORT) {
                    log::error!(
                        "Port {} is already in use. Another gptme-server instance may be running.",
                        GPTME_SERVER_PORT
                    );

                    // Show dialog to inform user about port conflict
                    let message = format!(
                        "Cannot start gptme-server because port {} is already in use.\n\n\
                        This usually means another gptme-server instance is already running.\n\n\
                        Please stop the existing gptme-server process and restart this application.",
                        GPTME_SERVER_PORT
                    );

                    MessageDialogBuilder::new(
                        app_handle.dialog().clone(),
                        "Port Conflict",
                        message
                    )
                    .kind(MessageDialogKind::Error)
                    .buttons(MessageDialogButtons::Ok)
                    .show(|_result| {
                        // Dialog closed, nothing to do
                    });

                    return;
                }

                // Determine CORS origin based on build mode
                let cors_origin = if cfg!(debug_assertions) {
                    "http://localhost:5701" // Dev mode
                } else {
                    "tauri://localhost" // Production mode
                };

                log::info!("Port {} is available, starting gptme-server with CORS origin: {}",
                          GPTME_SERVER_PORT, cors_origin);

                let sidecar_command = app_handle
                    .shell()
                    .sidecar("gptme-server")
                    .unwrap()
                    .args(["--cors-origin", cors_origin]);

                match sidecar_command.spawn() {
                    Ok((mut rx, child)) => {
                        log::info!(
                            "gptme-server started successfully with PID: {}",
                            child.pid()
                        );

                        // Store child process reference
                        if let Ok(mut child_ref) = child_process.lock() {
                            *child_ref = Some(child);
                        }

                        // Handle server output
                        tauri::async_runtime::spawn(async move {
                            while let Some(event) = rx.recv().await {
                                match event {
                                    tauri_plugin_shell::process::CommandEvent::Stdout(data) => {
                                        let output = String::from_utf8_lossy(&data);
                                        for line in output.lines() {
                                            if !line.trim().is_empty() {
                                                log::info!("[gptme-server] {}", line.trim());
                                            }
                                        }
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Stderr(data) => {
                                        let output = String::from_utf8_lossy(&data);
                                        for line in output.lines() {
                                            if !line.trim().is_empty() {
                                                log::warn!("[gptme-server] {}", line.trim());
                                            }
                                        }
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Error(error) => {
                                        log::error!("[gptme-server] Process error: {}", error);
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Terminated(
                                        payload,
                                    ) => {
                                        log::warn!(
                                            "[gptme-server] Process terminated with code: {:?}",
                                            payload.code
                                        );
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to start gptme-server: {}", e);
                    }
                }
            });

            // Store child process reference in app state for cleanup
            app.manage(child_for_cleanup);

            Ok(())
        })
        .on_window_event(|window, event| {
            // Handle window close event to cleanup server
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log::info!("Window close requested, cleaning up gptme-server...");

                {
                    let child_ref = window.state::<Arc<Mutex<Option<CommandChild>>>>().clone();
                    if let Ok(mut child) = child_ref.lock() {
                        if let Some(process) = child.take() {
                            log::info!("Attempting to terminate gptme-server process...");
                            match process.kill() {
                                Ok(_) => {
                                    log::info!("gptme-server process terminated successfully");
                                    // Give the process a moment to cleanup
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                },
                                Err(e) => {
                                    log::error!("Failed to terminate gptme-server: {}", e);
                                }
                            }
                        } else {
                            log::warn!("No gptme-server process found to terminate");
                        }
                    } else {
                        log::error!("Failed to acquire lock on child process reference");
                    };
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
