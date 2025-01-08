use std::error::Error;

use aws_sdk_sesv2 as ses;
use tera::{Context, Tera};

use crate::journal::JournalEntry;

pub fn render(
    tera: Tera,
    css: String,
    journal_entries: Vec<JournalEntry>,
) -> Result<ses::types::Content, Box<dyn Error>> {
    let mut context = Context::new();
    context.insert("style", &css);
    context.insert("journal_entries", &journal_entries);

    let content_str = tera.render("message.html", &context)?;

    let content = ses::types::Content::builder()
        .data(content_str)
        .build()
        .unwrap();

    Ok(content)
}
