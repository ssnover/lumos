use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use std::sync::Arc;
use std::error::Error;

use borealis::Aurora;
use helles::Server;
use lighthouse;
use tokio::runtime::Runtime;


fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting lumos...");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).unwrap();

    let (server, rx) = Server::new("/tmp/lumos.sock")?;
    let server_context = Server::start(server, running.clone());

    while running.load(Ordering::SeqCst) {
        match rx.try_recv() {
            Ok(cmd) => handle_cmd(cmd),
            Err(err) => match err {
                TryRecvError::Disconnected => {
                    eprintln!("Server context disconnected");
                    running.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }

    server_context.join().expect("Failed to join server thread");

    Ok(())
}

fn handle_cmd(cmd: String) {

    // TODO: This function is a hot mess
    // TODO: Come up with a [de]serialization of json commands
    // TODO: Come up with a [de]serialization of programming for buttons
    // TODO: Make an async based Hue crate
    // TODO: Use a struct and build some application state and real handling in for goodness sake
    let cmd_json: serde_json::Value = serde_json::from_str(&cmd).unwrap();
    if cmd_json["cmd"] == "ButtonEvent" {
        let button_id = cmd_json["args"]["ButtonId"].as_u64().unwrap();

        let aurora = Aurora::new(
            Ipv4Addr::new(192, 168, 1, 12),
            None,
            &"I8NTBbt5IsFhZ5yAuSaa38m9j70m4odx".to_string(),
        );

        let hue_bridge = lighthouse::bridge::Bridge::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7)), 
            "iNfKbUViaMmSITeJ5MPQlKA0-jsIJpsFK0nhjCRU".to_string()).unwrap();
    
        println!("Executing the cmd: {}", cmd);
        let mut runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .expect("Failed to build async runtime");
    
        runtime.block_on(async { 
            match button_id {
                111 => {
                    aurora.turn_on().await.unwrap();
                },
                113 => {
                    aurora.turn_off().await.unwrap();
                },
                _ => {
                    //
                },
            }
        });

        match button_id {
            116 => {
                // Hue lights on
                hue_bridge.state_to_multiple(vec![2, 3], vec![lighthouse::state!(on: true, bri: 255); 2]);
            },
            118 => {
                // Hue lights off
                hue_bridge.state_to_multiple(vec![2, 3], vec![lighthouse::state!(on: false); 2]);
            },
            _ => {
                //
            }
        }
    }
}