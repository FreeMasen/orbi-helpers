use clap::{Parser};
use lighter_ip_list::AttachedDevices;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(long, short, value_enum, default_value_t=OutputFormat::Table)]
    output_format: OutputFormat,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Simple,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let client = lighter_ip_list::get_client();

    let json = lighter_ip_list::get_attached_devices(&client)
        .await
        .unwrap();
    match args.output_format {
        OutputFormat::Simple => print_simple_output(&json),
        OutputFormat::Table => print_table_output(&json),
    }
}

fn print_simple_output(devs: &AttachedDevices) {
    for dev in &devs.devices {
        println!("{}: {}", dev.name, dev.ip);
    }
}

fn print_table_output(devs: &AttachedDevices) {
    let mut t = comfy_table::Table::new();
    t.set_header(vec!["Name", "Type", "Ip", "Satellite", "connection"]);
    for device in &devs.devices {
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
