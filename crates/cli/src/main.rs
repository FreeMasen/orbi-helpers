use clap::Parser;
use orbi_helpers::{AttachedDevices, Device};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(long, short, value_enum, default_value_t=OutputFormat::Table)]
    output_format: OutputFormat,
    #[arg(long, short = 'f')]
    device_fields: Vec<DeviceField>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Simple,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut args = Args::parse();
    let client = orbi_helpers::get_client();

    let json = orbi_helpers::get_attached_devices(&client)
        .await
        .unwrap();
    if args.device_fields.is_empty() {
        args.device_fields = vec![
            DeviceField::Name,
            DeviceField::Ip,
            DeviceField::Connection,
            DeviceField::Kind,
        ]
    }
    match args.output_format {
        OutputFormat::Simple => print_simple_output(&json),
        OutputFormat::Table => print_table_output(&json, &args.device_fields),
    }
}

fn print_simple_output(devs: &AttachedDevices) {
    for dev in &devs.devices {
        println!("{}: {}", dev.name, dev.ip);
    }
}

fn print_table_output(devs: &AttachedDevices, device_fields: &[DeviceField]) {
    let mut t = comfy_table::Table::new();
    if device_fields.is_empty() {
        t.set_header(vec!["Name", "Type", "Ip", "Satellite", "connection"]);
    } else {
        t.set_header(device_fields.into_iter().map(|f| {
            let r: &'static str = f.into();
            r
        }));
    }
    for device in &devs.devices {
        t.add_row(construct_row_from(device_fields, device));
    }
    println!("{t}");
}

fn construct_row_from<'a>(fields: &[DeviceField], device: &'a Device) -> Vec<&'a str> {
    let mut ret = Vec::new();
    for field in fields {
        match field {
            DeviceField::Name => ret.push(device.name.as_str()),
            DeviceField::Kind => ret.push(&device.kind),
            DeviceField::Ip => ret.push(&device.ip),
            DeviceField::Orbi => ret.push(&device.connected_orbi),
            DeviceField::Connection => ret.push(&device.connection_type),
            DeviceField::Mac => ret.push(&device.mac),
            DeviceField::Model => ret.push(&device.model),
        }
    }
    ret
}

#[derive(Clone, Copy, Debug, clap::ValueEnum, strum::IntoStaticStr)]
enum DeviceField {
    Mac,
    Kind,
    Model,
    Name,
    Ip,
    Orbi,
    Connection,
}
