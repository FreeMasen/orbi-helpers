
#[tokio::main]
async fn main() {
    env_logger::init();
    let client = lighter_ip_list::get_client();
    let json = lighter_ip_list::get_attached_devices(&client).await.unwrap();
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
    println!("{t}");
}
