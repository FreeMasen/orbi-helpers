use mime::Mime;
use reqwest::Client;
use warp::{Filter, Rejection};

use lighter_ip_list::{get_attached_devices, get_client};

#[tokio::main]
async fn main() {
    env_logger::init();
    let client = get_client();
    let attached_devices = warp::get()
        .and(warp::path("attached-devices"))
        .and(warp::header("accept"))
        .and_then({
            let client = client.clone();
            move |header: Mime| {
                let client = client.clone();
                async move {
                    let res = get_attached_devices(client.clone(), header)
                        .await
                        .map_err(|e| {
                            log::error!("Error: {e}");
                            reject::reject()
                        })?;
                    transform_response(res, header)
                }
            }
        });
    warp::serve(attached_devices.with(warp::log("orbi-helper")))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn transform_response(json: AttachedDevices, accept: Mime) -> Result<String, Rejection> {
    match accept.type_().as_str() {
        "application" if accept.subtype().as_str() == "json" => {
            serde_json::to_string(&json).map_err(|e| reject())
        }
        "text" if accept.subtype().as_str() == "plain" => {
            let mut t = comfy_table::Table::new();
            t.set_header(vec!["Name", "Type", "Ip", "Satellite", "connection"]);
            for device in &json.devices {
                t.add_row(vec![
                    &device.name,
                    &device.kind,
                    &device.ip,
                    &device.connected_orbi,
                    &device.connection_type,
                ]);
            }
            Ok(format!("{t}"))
        }
        _ => return Err(reject::reject()),
    }
}
