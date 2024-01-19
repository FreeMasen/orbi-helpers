# Orbi Helpers

A Rust crate for interacting with the Orbi Router over the local LAN


## Usage

### Installation

```shell
git clone https://github.com/FreeMasen/orbi-helpers \
&& cd ./orbi-helpers \
&& cargo build --release -p cli \
&& cp ./target/release/cli ~/.local/bin/orbi
```

### Configuration

A config.toml file should be installed in the platform config directory which can be found
in one of these directories

- Linux:   /home/$USER/.config/orbi-helper/config.toml
- Windows: %APPDATA%\rfm\orbi-helper/config.toml
- macOS:   /Users/$USER/Library/Application Support/com.rfm.orbi-helper/config.toml

This file should include the following 2 fields

- username
- password

Since Orbi doesn't allow for multiple user credentials, sadly this is the only way to
authenticate to the router.

Additionally a map of mac address/name to an over ride name can also be provided, for example

```toml
username = "..."
password = "..."

"AA:AA:AA:AA:AA:AA" = "Mom's Computer"
"some device name" = "Printer"
```

### Running

Currently there is just 1 functionality and that is to list the attached devices

```shell
Usage: orbi [OPTIONS]

Options:
  -o, --output-format <OUTPUT_FORMAT>  [default: table] [possible values: table, simple]
  -f, --device-fields <DEVICE_FIELDS>  [possible values: mac, kind, model, name, ip, orbi, connection]
  -h, --help                           Print help 
```

An example of the output might look something like this

```shell
+----------------------+--------------+------------+-------------------------+
| Name                 | Ip           | Connection | Kind                    |
+============================================================================+
| Living Room TV       | 192.168.1.1  | 2.4 GHz    | Wearable (Generic)      |
|----------------------+--------------+------------+-------------------------|
| Mom's Computer       | 192.168.1.2  | 5 GHz      | Laptop                  |
|----------------------+--------------+------------+-------------------------|
| Printer              | 192.168.1.3  | 5 GHz      | Network (Generic)       |
|----------------------+--------------+------------+-------------------------|
| SmartThings Hub      | 192.168.1.4  | Wired      | IoT (Generic)           |
|----------------------+--------------+------------+-------------------------|
| iPhone               | 192.168.1.5  | 5 GHz      | Smart Phone (Generic)   |
|----------------------+--------------+------------+-------------------------|
| Pixel                | 192.168.1.6  | 5 GHz      | Smart Phone (Generic)   |
+----------------------+--------------+------------+-------------------------+
```
