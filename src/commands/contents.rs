//! `exa contents` command implementation.

use serde_json::{Map, Value, json};

use crate::account;
use crate::cli::ContentsArgs;
use crate::error::{Error, Result};
use crate::output;

pub fn run(args: ContentsArgs) -> Result<()> {
    if !args.text
        && args.text_max_chars.is_none()
        && !args.highlights
        && args.highlights_max_chars.is_none()
        && args.highlights_query.is_none()
        && !args.summary
        && args.summary_query.is_none()
    {
        return Err(Error::invalid("Specify at least one content mode: --text, --highlights, or --summary."));
    }

    let resolved = account::resolve(args.auth.account.as_deref(), args.auth.api_key.as_deref())?;
    let client = resolved.client();

    let urls: Vec<String> = args.urls.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if urls.is_empty() {
        return Err(Error::invalid("At least one URL must be specified."));
    }

    let mut body = Map::new();
    body.insert("urls".into(), json!(urls));

    if args.text || args.text_max_chars.is_some() {
        if let Some(max_chars) = args.text_max_chars {
            let mut t = Map::new();
            t.insert("maxCharacters".into(), json!(max_chars));
            body.insert("text".into(), Value::Object(t));
        } else {
            body.insert("text".into(), json!(true));
        }
    }

    if args.highlights || args.highlights_max_chars.is_some() || args.highlights_query.is_some() {
        let mut h = Map::new();
        if let Some(m) = args.highlights_max_chars {
            h.insert("maxCharacters".into(), json!(m));
        }
        if let Some(q) = args.highlights_query {
            h.insert("query".into(), json!(q));
        }
        if !h.is_empty() {
            body.insert("highlights".into(), Value::Object(h));
        } else {
            body.insert("highlights".into(), json!(true));
        }
    }

    if args.summary || args.summary_query.is_some() {
        if let Some(q) = args.summary_query {
            let mut s = Map::new();
            s.insert("query".into(), json!(q));
            body.insert("summary".into(), Value::Object(s));
        } else {
            body.insert("summary".into(), json!(true));
        }
    }

    if let Some(max_age) = args.max_age_hours {
        body.insert("maxAgeHours".into(), json!(max_age));
    }
    if let Some(subpages) = args.subpages {
        body.insert("subpages".into(), json!(subpages));
    }
    if let Some(target) = args.subpage_target {
        let targets: Vec<String> =
            target.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if !targets.is_empty() {
            body.insert("subpageTarget".into(), json!(targets));
        }
    }

    let result = client.post("contents", &Value::Object(body))?;
    output::write(&result);
    Ok(())
}
