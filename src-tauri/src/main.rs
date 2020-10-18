#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cmd;

use music_speed::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Conf {
    path: String,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
enum Cmd {
    Analyze { payload: Conf },
    Test { payload: String },
}

fn main() {
    tauri::AppBuilder::new()
        .invoke_handler(|_webview, arg| {
            println!("Arg: {}", arg);
            use Cmd::*;
            match serde_json::from_str(arg) {
                Err(e) => Err(e.to_string()),
                Ok(command) => {
                    match command {
                        Test { payload } => {
                            println!("Test: {}", payload);
                        }
                        Analyze { payload } => {
                            analyse(Configuration {
                                file_path: &payload.path,
                                time_interval: 1000,
                                analysis_interval: 3000,
                                min_bpm: 80,
                                max_bpm: 160,
                                verbose: 1,
                            });
                        }
                    }
                    Ok(())
                }
            }
        })
        .build()
        .run();
}
