# systemd SES Notifier

A simple utility for automatic email alerts from failed systemd services via AWS SES

![Rust](https://img.shields.io/badge/rust-%23FF4300.svg?style=for-the-badge&logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-%23FCC624?style=for-the-badge&logo=linux&logoColor=black)
![AWS](https://img.shields.io/badge/AWS-%23232F3E.svg?style=for-the-badge&logo=amazonwebservices&logoColor=white)
![SES](https://img.shields.io/badge/SES-%23DD344C.svg?style=for-the-badge&logo=amazonsimpleemailservice&logoColor=white)

## Usage

### Manual Installation

```bash
cargo build --release
sudo install -m755 target/release/systemd-ses-notify /usr/local/bin/systemd-ses-notify
```

#### Config File

The config file location is different for system and user services

For system services: `/etc/systemd-ses-notify/config.toml`
For user services: `$HOME/.config/systemd-ses-notify/config.toml`

```toml
# journalctl lookback, in seconds [optional (default 500)]
lookback = 500

send_from = "Alerts <alerts@email.com>"
send_to = [ "person@email.com" ]

# path to HTML message template
template = "/path/to/template.html"

# stylesheet for message [optional]
css = "/path/to/stylesheet.css"
```

#### Service File

This can go in one of a few places, I'd recommend the following

For system services: `/usr/local/lib/systemd/system/systemd-ses-notify@.service`
For user services: `/usr/local/lib/systemd/user/systemd-ses-notify@.service`

```desktop
[Unit]
Description=Sends a notification about a failed systemd unit to AWS SES
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/systemd-ses-notify %i
```

#### AWS Environment Setup

You can specify AWS credentials with a service override, the "recommended" is with `systemctl`

```bash
sudo systemctl edit systemd-ses-notify@.service
# or
systemctl --user edit systemctl-ses-notify@.service
```

Then add the following, with you IAM user credentials

```desktop
Environment="AWS_REGION="
Environment="AWS_SECRET_ACCESS_KEY="
Environment="AWS_ACCESS_KEY_ID="
```

#### Failure Override File

This can go in one of a few places, I'd recommend the following

For system services: `/usr/local/lib/systemd/system/service.d/failure-override.conf`
For user services: `/usr/local/lib/systemd/user/service.d/failure-override.conf`

```desktop
[Unit]
OnFailure=systemd-ses-notify@%n
```

A second file is also needed to avoid a loop if the failure service itself fails. We can create an empty file at the following locations

For system services: `/usr/local/lib/systemd/system/systemd-ses-notify@.service.d/failure-override.conf`
For user services: `/usr/local/lib/systemd/user/systemd-ses-notify@.service.d/failure-override.conf`

#### Message template

A basic message template can be found in the [templates directory](https://github.com/rivnakm/systemd-ses-notify/blob/develop/templates/example.html).
More specific info can be found in the [Tera](https://keats.github.io/tera/) docs.
