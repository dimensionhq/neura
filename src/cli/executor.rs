use crate::commands;

use super::validator::ValidatedOptions;
use miette::Result;

// Execute the command passed in
pub async fn execute(options: ValidatedOptions) -> Result<()> {
    match options {
        ValidatedOptions::Init {} => commands::init::execute().await,
        ValidatedOptions::Watch {} => commands::watch::execute().await,
        ValidatedOptions::None => commands::watch::execute().await,
    }
}
