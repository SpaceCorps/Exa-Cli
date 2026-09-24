//! `exa find-similar` command implementation.

use serde_json::{Map, Value, json};

use crate::account;
use crate::cli::FindSimilarArgs;
use crate::error::Result;
use crate::output;

pub fn run(args: FindSimilarArgs) -> Result<()> {
    let resolved = account::resolve(args.auth.account.as_deref(), args.auth.api_key.as_deref())?;
    let client = resolved.client();

    let contents = build_contents(&args);

    let mut body = Map::new();
    body.insert("url".into(), json!(args.url));

    if let Some(num) = args.num_results {
        body.insert("numResults".into(), json!(num));
    }
    if let Some(start) = args.start_published_date {
        body.insert("startPublishedDate".into(), json!(start));
    }
    if let Some(end) = args.end_published_date {
        body.insert("endPublishedDate".into(), json!(end));
    }

    if let Some(inc) = args.include_domains {
        let domains = split_domains(&inc);
        if !domains.is_empty() {
            body.insert("includeDomains".into(), json!(domains));
        }
    }
    if let Some(exc) = args.exclude_domains {
        let domains = split_domains(&exc);
        if !domains.is_empty() {
            body.insert("excludeDomains".into(), json!(domains));
        }
    }

    if !contents.is_empty() {
        body.insert("contents".into(), Value::Object(contents));
    }

    let result = client.post("findSimilar", &Value::Object(body))?;
    output::write(&result);
    Ok(())
}

fn build_contents(args: &FindSimilarArgs) -> Map<String, Value> {
    let mut contents = Map::new();

    if args.highlights || args.highlights_max_chars.is_some() {
        if let Some(max_chars) = args.highlights_max_chars {
            let mut h = Map::new();
            h.insert("maxCharacters".into(), json!(max_chars));
            contents.insert("highlights".into(), Value::Object(h));
        } else {
            contents.insert("highlights".into(), json!(true));
        }
    }

    if args.text || args.text_max_chars.is_some() {
        if let Some(max_chars) = args.text_max_chars {
            let mut t = Map::new();
            t.insert("maxCharacters".into(), json!(max_chars));
            contents.insert("text".into(), Value::Object(t));
        } else {
            contents.insert("text".into(), json!(true));
        }
    }

    if args.summary {
        contents.insert("summary".into(), json!(true));
    }

    contents
}

fn split_domains(domains: &str) -> Vec<String> {
    domains.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}
