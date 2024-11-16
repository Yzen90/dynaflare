# DynaFlare

## Simple IPv4 DDNS client for Cloudflare

### 1. Obtaining DynaFlare

#### Building from source
```shell
git clone https://github.com/Yzen90/dynaflare.git
cd dynaflare
cargo build --release
```
#### Crates.io
```shell
cargo install dynaflare
```

#### Pre-build binaries
- [Linux aarch64](http://example.com)
- [Linux x86_64](http://example.com)
- [Windows x86_64](http://example.com)

### 2. Configuration

DynaFlare looks in the current working directory for a `configuration.toml` file with the following fields:

- `interval` (Optional): If not provided, dynaflare will check and update de provided records only once and then exit. Otherwise dynaflare will keep runing and check the public ip and update the dns record if needed at this interval.
- `zone_id` (Required): ID of the Clouldflare Zone that contains the provided dns records.
- `api_token` (Required): Cloudflare API token with DNS read and edit permissions for the provided zone.
- `records` (Required): DNS records to set to the current public IP address.

> `configuration.toml`
```toml
interval = "1min 30s"
zone_id = "cloudflare zone id"
api_token = "cloudflare api token"
records = ["dynamic.example.com"]
```
