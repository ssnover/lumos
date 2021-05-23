use std::error::Error;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use borealis::client::Aurora;
use futures::stream::StreamExt;
use paho_mqtt as mqtt;

mod lumos_msgs;
use lumos_msgs::ButtonEvent::{ButtonEvent, EventType};
use protobuf::Message;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting lumos...");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri("tcp://localhost:1883")
        .client_id("lumos")
        .finalize();
    let mut mqtt_client = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        eprintln!("Error building mqtt client: {:?}", err);
        std::process::exit(1);
    });

    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            eprintln!("Error building the async tokio runtime: {:?}", err);
            std::process::exit(1);
        });
    runtime.block_on(async {
        let mut rx = mqtt_client.get_stream(10);
        let connect_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .finalize();
        println!("Making connection to the MQTT broker...");
        mqtt_client.connect(connect_opts).await.unwrap_or_else(|err| {
            eprintln!("Unable to connect to mqtt broker: {:?}", err);
            std::process::exit(1);
        });
        
        println!("Connection established, subscribing to topics.");
        mqtt_client.subscribe("/lumos/events", 1).await.unwrap_or_else(|err| {
            eprintln!("Unable to create mqtt subscriptions: {:?}", err);
            std::process::exit(1);
        });

        while let Some(msg) = rx.next().await {
            if let Some(msg) = msg {
                match msg.topic() {
                    "/lumos/events" => {
                        handle_event(msg.payload()).await;
                    }
                    _ => {
                        eprintln!("Received mqtt message for unhandled topic: {}", msg.topic());
                    }
                }
            }
        }
    });

    Ok(())
}

async fn handle_event(payload: &[u8]) {
    if let Ok(button_event) = ButtonEvent::parse_from_bytes(payload) {
        if button_event.event == EventType::EVENT_BUTTON_RELEASE {
            let aurora = Aurora::new(
                Ipv4Addr::new(192, 168, 1, 12),
                None,
                &"I8NTBbt5IsFhZ5yAuSaa38m9j70m4odx".to_string(),
            );

            match button_event.button_id {
                111 => {
                    aurora.turn_on().await;
                }
                113 => {
                    aurora.turn_off().await;
                }
                _ => {
                    eprintln!("Received button event for unknown button {}, dropping...", button_event.button_id);
                }
            }
        }
    }
}
