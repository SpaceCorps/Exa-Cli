//! `exa answer` command implementation.

use serde_json::{Map, Value, json};

use crate::account;
use crate::cli::AnswerArgs;
use crate::error::Result;
use crate::output;

pub fn run(args: AnswerArgs) -> Result<()> {
    let resolved = account::resolve(args.auth.account.as_deref(), args.auth.api_key.as_deref())?;
    let client = resolved.client();

    let mut body = Map::new();
    body.insert("query".into(), json!(args.query));

    if args.text {
        body.insert("text".into(), json!(true));
    }

    let result = client.post("answer", &Value::Object(body))?;
    output::write(&result);
    Ok(())
}
