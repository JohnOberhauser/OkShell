use clap::Parser;
use std::error::Error;

use okshell_cli_style;
use okshell_core;
use okshell_logging;

#[derive(Parser)]
#[command(
    name = "okshell",
    version,
    about = "OkShell desktop shell",
    styles = okshell_cli_style::get_styles(),
)]
struct Args {}

fn main() -> Result<(), Box<dyn Error>> {
    let _args = Args::parse();

    okshell_logging::init("okshell");

    okshell_core::run()?;

    Ok(())
}
