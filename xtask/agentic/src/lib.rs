pub mod autonomous_pretty_print;
pub mod claude_session;
pub mod pretty_print;

use anyhow::Result;
use bpaf::Bpaf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum AgenticCommand {
    /// Generate ToTokens implementations for pretty printing using AI
    #[bpaf(command("pretty-print-impls"))]
    PrettyPrintImpls,
    /// Run autonomous pretty print implementation generator
    #[bpaf(command("autonomous-pretty-print"))]
    AutonomousPrettyPrint,
}

pub fn run_agentic_task(cmd: AgenticCommand) -> Result<()> {
    match cmd {
        AgenticCommand::PrettyPrintImpls => pretty_print::run_pretty_print_generator(),
        AgenticCommand::AutonomousPrettyPrint => {
            autonomous_pretty_print::run_autonomous_pretty_print_generator()
        }
    }
}
