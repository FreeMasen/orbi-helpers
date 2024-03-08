use clap::{Parser, Subcommand};
use orbi_helpers::{AttachedDevices, Device};

// #[derive(Clone, Debug, Parser)]
// pub struct Args {
//     #[arg(long, short, value_enum, default_value_t=OutputFormat::Table)]
//     output_format: OutputFormat,
//     #[arg(long, short = 'f')]
//     device_fields: Vec<DeviceField>,
// }

#[derive(Clone, Debug, Parser)]
enum Commands {
    Devices {
        #[arg(long, short, value_enum, default_value_t=OutputFormat::Table)]
        output_format: OutputFormat,
        #[arg(long, short = 'f')]
        device_fields: Vec<DeviceField>,
    },
    #[clap(subcommand)]
    Config(ConfigCommands)
}

#[derive(Clone, Subcommand, Debug)]
enum ConfigCommands {
    Dump {
        #[clap(short = 'p', long)]
        show_password: bool,
    },
    GetPath,
    SetUsername {
        username: String,
    },
    SetPassword {
        password: String,
    },
    SetOverride {
        /// The MAC or Name of the device
        name: String,
        /// What you want it to display as
        replacement: String,
    },
    ClearOverride {
        name: String,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Simple,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Commands::parse();
    match args {
        Commands::Devices { output_format, device_fields } => devices_main(device_fields, output_format).await,
        Commands::Config(command) => config_main(command).await,
    }
}

async fn config_main(command: ConfigCommands) {
    match command {
        ConfigCommands::Dump {show_password} => {
            let config = orbi_helpers::read_config().await.unwrap();
            println!("username: {}", config.username);
            let pw = if show_password {
                config.password
            } else {
                "*".repeat(config.password.len())
            };
            println!("password: {pw}");
            println!("overrides:");
            for (name, over) in config.device_name_overrides {
                println!("  {name} -> {over}");
            }
        }
        ConfigCommands::GetPath => {
            let path = orbi_helpers::find_config_path().unwrap();
            println!("{}", path.display());
        }
        ConfigCommands::SetOverride { name, replacement } => {
            orbi_helpers::set_config_name_override(name, Some(replacement)).await.unwrap();
        }
        ConfigCommands::ClearOverride { name } => {
            orbi_helpers::set_config_name_override(name, None).await.unwrap();
        }
        ConfigCommands::SetPassword { password } => orbi_helpers::set_config_password(&password).await.unwrap(),
        ConfigCommands::SetUsername { username } => orbi_helpers::set_config_username(&username).await.unwrap(),
    }
}

async fn devices_main(mut device_fields: Vec<DeviceField>,output_format: OutputFormat,) {
    let client = orbi_helpers::get_client();

    let json = orbi_helpers::get_attached_devices(&client)
        .await
        .unwrap();
    if device_fields.is_empty() {
        device_fields = vec![
            DeviceField::Name,
            DeviceField::Ip,
            DeviceField::Connection,
            DeviceField::Kind,
        ]
    }
    match output_format {
        OutputFormat::Simple => print_simple_output(&json),
        OutputFormat::Table => print_table_output(&json, &device_fields),
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
