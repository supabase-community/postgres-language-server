use anyhow::Result;
use xshell::{cmd, Shell};

use crate::flags::{Agentic, AgenticCmd};

pub fn run(cmd: Agentic, sh: &Shell) -> Result<()> {
    match cmd.subcommand {
        AgenticCmd::PrettyPrintImpls(_) => {
            println!("Running agentic pretty print implementation generator...");
            cmd!(sh, "cargo run -p xtask_agentic").run()?;
        }
    }
    Ok(())
}
