# DynaFlare
#### Simple IPv4 DDNS client for Cloudflare. Works on the Raspberry Pi!


### 1. Obtaining DynaFlare

#### Build from source
```shell
git clone https://github.com/Yzen90/dynaflare.git
cd dynaflare
cargo build --release
```

#### Install crate
```shell
cargo install dynaflare
```

#### Download a pre-built binary
- [Linux aarch64](https://github.com/Yzen90/dynaflare/releases/latest/download/dynaflare-aarch64-linux.zip)
- [Linux x86_64](https://github.com/Yzen90/dynaflare/releases/latest/download/dynaflare-x86_64-linux.zip)
- [Windows x86_64](https://github.com/Yzen90/dynaflare/releases/latest/download/dynaflare-x86_64-windows.zip)


### 2. Configuration

DynaFlare looks in the current working directory for a `configuration.toml` file with the following fields:

- `interval` (_Optional_): String with one or more pairs of a positive integer immediately followed by 'days', 'h', 'min', 's', 'ms', 'μs' or 'ns'. If not provided, dynaflare will check and update de provided records only once and then exit. Otherwise dynaflare will keep runing and continuously check at this interval for public ip changes and update the dns records when necessary.

- `zone_id` (_Required_): ID of the Clouldflare Zone that contains the provided dns records.

- `api_token` (_Required_): Cloudflare API token with DNS read and edit permissions for the provided zone.

- `records` (_Required_): DNS records to update or create with the current public IPv4 address.

- `count_repeated_errors` (_Optional_): Defaults to `false`. If `true`, consecutive identical errors are shown only the first time, then the additional error count is shown until there is a successful request or a different error.

- `log_level` (_Optional_): The log level filter, defaults to `INFO`. Valid values are `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, or `OFF`.

#### Example `configuration.toml`

```toml
interval = "1min 30s"
zone_id = "cloudflare zone id here"
api_token = "cloudflare api token here"
records = ["dynamic.example.com"]
count_repeated_errors = true
```


### 3. systemd service

A systemd service unit file template for DynaFlare is provided in the repository that can be used as follows to install a systemd service:

```shell
export DYNAFLARE_USER=dynaflare # User that the process will be executed as
export DYNAFLARE_WDIR=/usr/local/etc/dynaflare # Directory where configuration.toml is located
export DYNAFLARE_EXEC=/usr/local/bin/dynaflare # Path of dynaflare executable file

envsubst < dynaflare.service | sudo tee /etc/systemd/system/dynaflare.service

sudo systemctl start dynaflare
sudo systemctl status dynaflare
sudo systemctl enable dynaflare
```

#### Example systemd unit file
```ini
[Unit]
Description=DynaFlare, a simple DDNS client for Cloudflare
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