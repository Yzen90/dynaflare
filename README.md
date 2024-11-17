# DynaFlare
## Simple IPv4 DDNS client for Cloudflare


### 1. Obtaining DynaFlare

#### Build from source
```shell
git clone https://github.com/Yzen90/dynaflare.git
cd dynaflare
cargo build --release
```

#### Crate
```shell
cargo install dynaflare
```

#### Pre-built binaries
- [Linux aarch64](https://github.com/Yzen90/dynaflare/releases/download/v1.0.0/dynaflare-v1.0.0-aarch64-linux.zip)
- [Linux x86_64](https://github.com/Yzen90/dynaflare/releases/download/v1.0.0/dynaflare-v1.0.0-x86_64-linux.zip)
- [Windows x86_64](https://github.com/Yzen90/dynaflare/releases/download/v1.0.0/dynaflare-v1.0.0-x86_64-windows.zip)


### 2. Configuration

DynaFlare looks in the current working directory for a `configuration.toml` file with the following fields:

- `interval` (Optional): If not provided, dynaflare will check and update de provided records only once and then exit. Otherwise dynaflare will keep runing and continuosly check at this interval for public ip changes and update the dns record when necessary.
- `zone_id` (Required): ID of the Clouldflare Zone that contains the provided dns records.
- `api_token` (Required): Cloudflare API token with DNS read and edit permissions for the provided zone.
- `records` (Required): DNS records to set to the current public IP address.

#### Example `configuration.toml`

```toml
interval = "1min 30s"
zone_id = "cloudflare zone id here"
api_token = "cloudflare api token here"
records = ["dynamic.example.com"]
```

### 3. systemd service

A systemd service unit file template for DynaFlare is provided in the repository that can be used as follows to install a systemd service:

```shell
export DYNAFLARE_USER=dynaflare # User that the process will be executed as
export DYNAFLARE_WDIR=/usr/local/etc/dynaflare # Directory where configuration.toml is located
export DYNAFLARE_EXEC=/usr/local/bin/dynaflare # Path of dynaflare executable file

sudo envsubst < dynaflare.service > /etc/systemd/system/dynaflare.service

sudo systemctl start dynaflare
sudo systemctl status dynaflare
sudo systemctl enable dynaflare
```

#### Example systemd unit file
```ini
[Unit]
Description=DynaFlare simple DDNS client for Cloudflare
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=dynaflare
WorkingDirectory=/usr/local/etc/dynaflare
ExecStart=/usr/local/bin/dynaflare

[Install]
WantedBy=multi-user.target
```