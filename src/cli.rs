//! The command tree. One variant per command; `commands` does the work.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "exa",
    version,
    about = "CLI for the Exa AI search API — search, find similar, extract contents, and answers",
    after_help = "An LLM agent should start with: exa agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone, Default, Debug)]
pub struct AuthArgs {
    /// Account to run against (see 'exa accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT", global = true)]
    pub account: Option<String>,

    /// Exa API key (or set EXA_API_KEY env var)
    #[arg(long, value_name = "KEY", env = "EXA_API_KEY", global = true)]
    pub api_key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search the web with natural language
    Search(SearchArgs),

    /// Extract clean content from URLs
    Contents(ContentsArgs),

    /// Get an LLM-generated answer with citations
    Answer(AnswerArgs),

    /// Find pages similar to a given URL
    #[command(name = "find-similar")]
    FindSimilar(FindSimilarArgs),

    /// Log in with an Exa API key (opens browser to copy key)
    Login(LoginArgs),

    /// Manage Exa accounts and their API keys
    #[command(subcommand)]
    Accounts(AccountsCommand),

    /// Print the operating manual for an LLM agent driving this CLI
    #[command(name = "agent-readme")]
    AgentReadme,
}

// ---------------------------------------------------------------------------------------------
// search

#[derive(Args, Clone, Debug)]
pub struct SearchArgs {
    /// Natural language search query
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Number of results (1-100, default 10)
    #[arg(long = "num", value_name = "COUNT")]
    pub num_results: Option<u32>,

    /// Search type: auto, fast, instant, deep-lite, deep, deep-reasoning
    #[arg(long = "type", value_name = "TYPE")]
    pub search_type: Option<String>,

    /// Category: company, people, news, research paper, personal site, financial report
    #[arg(long, value_name = "CATEGORY")]
    pub category: Option<String>,

    /// Comma-separated domain whitelist
    #[arg(long, value_name = "DOMAINS")]
    pub include_domains: Option<String>,

    /// Comma-separated domain blacklist
    #[arg(long, value_name = "DOMAINS")]
    pub exclude_domains: Option<String>,

    /// Minimum published date (ISO 8601)
    #[arg(long = "start-date", value_name = "DATE")]
    pub start_published_date: Option<String>,

    /// Maximum published date (ISO 8601)
    #[arg(long = "end-date", value_name = "DATE")]
    pub end_published_date: Option<String>,

    /// Return query-relevant excerpts
    #[arg(long)]
    pub highlights: bool,

    /// Max characters for highlights (default 4000)
    #[arg(long = "highlights-chars", value_name = "COUNT")]
    pub highlights_max_chars: Option<u32>,

    /// Return full page text as markdown
    #[arg(long)]
    pub text: bool,

    /// Max characters for text
    #[arg(long = "text-chars", value_name = "COUNT")]
    pub text_max_chars: Option<u32>,

    /// Return LLM-generated summary
    #[arg(long)]
    pub summary: bool,

    /// Two-letter ISO country code
    #[arg(long, value_name = "CODE")]
    pub user_location: Option<String>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// contents

#[derive(Args, Clone, Debug)]
pub struct ContentsArgs {
    /// Comma-separated URLs to extract content from
    #[arg(value_name = "URLS")]
    pub urls: String,

    /// Return full page text as markdown
    #[arg(long)]
    pub text: bool,

    /// Max characters for text
    #[arg(long = "text-chars", value_name = "COUNT")]
    pub text_max_chars: Option<u32>,

    /// Return query-relevant excerpts
    #[arg(long)]
    pub highlights: bool,

    /// Max characters for highlights
    #[arg(long = "highlights-chars", value_name = "COUNT")]
    pub highlights_max_chars: Option<u32>,

    /// Custom query to direct highlight selection
    #[arg(long, value_name = "QUERY")]
    pub highlights_query: Option<String>,

    /// Return LLM-generated summary
    #[arg(long)]
    pub summary: bool,

    /// Custom query for the summary
    #[arg(long, value_name = "QUERY")]
    pub summary_query: Option<String>,

    /// Max cache age in hours (0=always livecrawl, -1=cache only)
    #[arg(long = "max-age", value_name = "HOURS")]
    pub max_age_hours: Option<i32>,

    /// Number of subpages to crawl per URL
    #[arg(long, value_name = "COUNT")]
    pub subpages: Option<u32>,

    /// Comma-separated keywords to prioritize subpage selection
    #[arg(long = "subpage-target", value_name = "KEYWORDS")]
    pub subpage_target: Option<String>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// answer

#[derive(Args, Clone, Debug)]
pub struct AnswerArgs {
    /// The question to answer
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Include full text content in citations
    #[arg(long)]
    pub text: bool,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// find-similar

#[derive(Args, Clone, Debug)]
pub struct FindSimilarArgs {
    /// Reference URL to find similar pages for
    #[arg(value_name = "URL")]
    pub url: String,

    /// Number of results (1-100, default 10)
    #[arg(long = "num", value_name = "COUNT")]
    pub num_results: Option<u32>,

    /// Comma-separated domain whitelist
    #[arg(long, value_name = "DOMAINS")]
    pub include_domains: Option<String>,

    /// Comma-separated domain blacklist
    #[arg(long, value_name = "DOMAINS")]
    pub exclude_domains: Option<String>,

    /// Minimum published date (ISO 8601)
    #[arg(long = "start-date", value_name = "DATE")]
    pub start_published_date: Option<String>,

    /// Maximum published date (ISO 8601)
    #[arg(long = "end-date", value_name = "DATE")]
    pub end_published_date: Option<String>,

    /// Return query-relevant excerpts
    #[arg(long)]
    pub highlights: bool,

    /// Max characters for highlights
    #[arg(long = "highlights-chars", value_name = "COUNT")]
    pub highlights_max_chars: Option<u32>,

    /// Return full page text as markdown
    #[arg(long)]
    pub text: bool,

    /// Max characters for text
    #[arg(long = "text-chars", value_name = "COUNT")]
    pub text_max_chars: Option<u32>,

    /// Return LLM-generated summary
    #[arg(long)]
    pub summary: bool,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// login

#[derive(Args, Clone, Debug)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Exa API key (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin, e.g. `pbpaste | exa login --api-key-stdin`
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API keys page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the key on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the key without calling the API to check it first
    #[arg(long)]
    pub no_verify: bool,
}

// ---------------------------------------------------------------------------------------------
// accounts

#[derive(Subcommand, Debug)]
pub enum AccountsCommand {
    /// Add an account and store its API key in the OS keystore
    Add {
        /// Short name for this account, used as --account elsewhere
        name: String,
        /// Exa API key (prompted for, without echo, if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,
        /// Read the API key from stdin, e.g. `pbpaste | exa accounts add work --api-key-stdin`
        #[arg(long)]
        api_key_stdin: bool,
        /// Replace the key on an account that already exists
        #[arg(long)]
        force: bool,
        /// Store the key without calling the API to check it first
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API once per account to check validity
        #[arg(long)]
        check: bool,
    },
    /// Check that an account's stored key still works
    Test {
        /// Account name
        name: String,
    },
    /// Remove an account and delete its stored key
    Remove {
        /// Account name
        name: String,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_tree_is_valid() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}
