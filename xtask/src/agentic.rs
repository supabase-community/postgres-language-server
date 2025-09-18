use anyhow::Result;
use xshell::{cmd, Shell};

use crate::flags::Agentic;

pub fn run(cmd: Agentic, sh: &Shell) -> Result<()> {
    println!("Running autonomous pretty print implementation generator...");
    if cmd.forever {
        cmd!(sh, "cargo run -p xtask_agentic -- --forever").run()?;
    } else {
        cmd!(sh, "cargo run -p xtask_agentic").run()?;
    }
    Ok(())
}
