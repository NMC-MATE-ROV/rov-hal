use anyhow::bail;
use axum::{Router, routing::{get, post}, extract::{Extension, WebSocketUpgrade, Json, Path}, response::IntoResponse};
use axum::extract::ws::{WebSocket, Message};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;
use futures::{StreamExt, SinkExt};

mod hal;
mod system;
use hal::devices::device::{RawDevice, create_gpio_devices, DevicesRegistry};
use system::System;
use std::collections::HashMap;

#[derive(Deserialize)]
struct GenericWsMessage {
    device: String,
    cmd: String,
    params: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let gpio = rppal::gpio::Gpio::new()?;

    // Read devices configuration (expecting ./devices.json)
    if !fs::exists("/home/rov/.config/devices.json")? {
        bail!("ROV: devices.json file does not exist");
    }
    let cfg = std::fs::read_to_string("/home/rov/.config/devices.json").unwrap_or_else(|_| "[]".to_string());

    let raw_devices: Vec<RawDevice> = serde_json::from_str(&cfg)?;
    // Always create system device; optionally override with config settings
    let mut map = create_gpio_devices(&raw_devices, &gpio)?;

    // Create system device (always exists)
    let sys_settings: Option<&Value> = raw_devices.iter()
        .find(|d| d.id == "system")
        .and_then(|d| d.params.as_ref());

    // Pass all devices map to system device for list_devices command
    let devices_for_system: HashMap<String, String> = raw_devices.iter()
        .map(|d| (d.id.clone(), d.device_type.clone()))
        .collect();

    if let Some(settings) = sys_settings {
        map.insert("system".to_string(), Arc::new(Mutex::new(Box::new(System::with_settings(devices_for_system.clone(), settings)))));
    } else {
        map.insert("system".to_string(), Arc::new(Mutex::new(Box::new(System::with_settings(devices_for_system, &serde_json::json!({})) ))));
    }

    let devices_registry: DevicesRegistry = Arc::new(Mutex::new(map));

    // Define websocket route
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/device/{device}/{cmd}", post(rest_device_handler))
        .layer(Extension(devices_registry.clone()));

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, Extension(devices): Extension<DevicesRegistry>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, devices))
}

#[axum::debug_handler]
async fn rest_device_handler(Path((device, cmd)): Path<(String, String)>, Extension(devices): Extension<DevicesRegistry>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let params = &body;

    let map = devices.lock().await;
    if let Some(shared_dev) = map.get(&device) {
        let mut dev = shared_dev.lock().await;
        match dev.handle(&cmd, params) {
            Ok(resp) => (axum::http::StatusCode::OK, axum::Json(resp)).into_response(),
            Err(e) => (axum::http::StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({"error": e}))).into_response(),
        }
    } else {
        (axum::http::StatusCode::NOT_FOUND, axum::Json(serde_json::json!({"error": format!("unknown device: {}", device)}))).into_response()
    }
}

async fn handle_socket(socket: WebSocket, devices: DevicesRegistry) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<GenericWsMessage>(&text) {
                    Ok(req) => {
                        let map = devices.lock().await;
                        if let Some(shared_dev) = map.get(&req.device) {
                            let mut dev = shared_dev.lock().await;
                            let default_params = serde_json::json!({});
                            let params = req.params.as_ref().unwrap_or(&default_params);
                            match dev.handle(&req.cmd, params) {
                                Ok(resp) => {
                                    let _ = sender.send(Message::Text(resp.to_string().into())).await;
                                }
                                Err(e) => {
                                    let _ = sender.send(Message::Text(json!({"error": e}).to_string().into())).await;
                                }
                            }
                        } else {
                            let _ = sender.send(Message::Text(json!({"error": format!("unknown device: {}", req.device)}).to_string().into())).await;
                        }
                    }
                    Err(e) => {
                        let _ = sender.send(Message::Text(json!({"error": format!("invalid json: {}", e)}).to_string().into())).await;
                    }
                }
            }
            Message::Close(_) => {
                let _ = sender.send(Message::Close(None)).await;
                break;
            }
            _ => {}
        }
    }
}
