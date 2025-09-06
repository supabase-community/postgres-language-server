use anyhow::Result;
use xtask_agentic::run_autonomous_pretty_print_generator;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let forever = args.iter().any(|arg| arg == "--forever");

    run_autonomous_pretty_print_generator(forever)
}
