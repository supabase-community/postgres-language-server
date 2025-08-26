use anyhow::Result;
use xtask_agentic::{run_agentic_task, AgenticCommand};

fn main() -> Result<()> {
    // For now, just run the pretty print implementation
    let cmd = AgenticCommand::PrettyPrintImpls;
    run_agentic_task(cmd)
}
