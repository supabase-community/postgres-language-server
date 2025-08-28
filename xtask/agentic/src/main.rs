use anyhow::Result;
use xtask_agentic::{run_agentic_task, AgenticCommand};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let cmd = if args.len() > 1 && args[1] == "autonomous" {
        AgenticCommand::AutonomousPrettyPrint
    } else {
        AgenticCommand::PrettyPrintImpls
    };

    run_agentic_task(cmd)
}
