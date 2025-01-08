use std::{path::PathBuf, time::SystemTime};

use aws_sdk_sesv2 as ses;

use aws_config::BehaviorVersion;
use chrono::DateTime;
use clap::Parser;
use config::Config;
use journal::{get_journal_entries, JournalEntry};
use message::render;
use tera::Tera;

mod config;
mod journal;
mod message;

/// Send an alert email for a failed systemd unit via AWS SES
#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Systemd unit
    unit: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let is_root = whoami::username() == "root";
    let config_path = if is_root {
        PathBuf::from("/etc/systemd-ses-notify/config.toml")
    } else {
        let user_conf_dir = dirs::config_dir().unwrap();
        user_conf_dir.join("systemd-ses-notify/config.toml")
    };

    let app_config = config::load_config(config_path.as_path()).expect("unable to load config");

    let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    let ses_client = ses::Client::new(&aws_config);

    let since = DateTime::from(SystemTime::now()) - app_config.lookback;
    let journal_entries = get_journal_entries(args.unit.as_str(), !is_root, since)
        .expect("failed to get journal entries");

    let message = build_message(args.unit.as_str(), app_config.clone(), journal_entries);
    let content = ses::types::EmailContent::builder().simple(message).build();

    let destination = ses::types::Destination::builder()
        .set_to_addresses(Some(app_config.send_to))
        .build();

    let _req = ses_client
        .send_email()
        .from_email_address(app_config.send_from)
        .destination(destination)
        .content(content)
        .send()
        .await
        .expect("SES Send failed");
}

fn build_message(
    unit: &str,
    config: Config,
    journal_entries: Vec<JournalEntry>,
) -> ses::types::Message {
    let mut tera = Tera::default();
    tera.add_template_file(config.template, Some("message.html"))
        .unwrap();

    let css = match config.css {
        Some(path) => std::fs::read_to_string(path).expect("failed to read css"),
        None => String::new(),
    };
    let html = render(tera, css, journal_entries).expect("failed to render message template");

    let body = ses::types::Body::builder().html(html).build();

    let subject = ses::types::Content::builder()
        .data(format!(
            "[Alert] {} failed on {}",
            unit,
            whoami::fallible::hostname().unwrap_or(String::from("Unknown"))
        ))
        .build()
        .unwrap();

    ses::types::Message::builder()
        .subject(subject)
        .body(body)
        .build()
}
