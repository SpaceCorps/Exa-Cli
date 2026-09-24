//! Subcommand implementations and dispatch.

pub mod accounts;
pub mod answer;
pub mod contents;
pub mod find_similar;
pub mod login;
pub mod search;

use crate::cli::Command;
use crate::error::Result;
use crate::readme;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Search(args) => search::run(args),
        Command::Contents(args) => contents::run(args),
        Command::Answer(args) => answer::run(args),
        Command::FindSimilar(args) => find_similar::run(args),
        Command::Login(args) => login::run(args),
        Command::Accounts(cmd) => accounts::run(cmd),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}
