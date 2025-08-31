use anyhow::Result;
use xshell::{cmd, Shell};

use crate::flags::Agentic;

pub fn run(_cmd: Agentic, sh: &Shell) -> Result<()> {
    println!("Running autonomous pretty print implementation generator...");
    cmd!(sh, "cargo run -p xtask_agentic").run()?;
    Ok(())
}
