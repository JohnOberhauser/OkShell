use clap::Parser;
use okshellctl::app::{Cli, Commands};
use okshellctl::bus::{bus_command, bus_command_with_arg};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Quit => { bus_command("Quit").await?; }
        Commands::Inspect => { bus_command("Inspect").await?; }
        Commands::Menu { command } => okshellctl::subcommands::menu::execute(command).await?,
        Commands::Bar { command } => okshellctl::subcommands::bar::execute(command).await?,
        Commands::Audio { command } => okshellctl::subcommands::audio::execute(command).await?,
        Commands::Brightness { command } => okshellctl::subcommands::brightness::execute(command).await?,
        Commands::SetWallpaper { path } => {
            bus_command_with_arg("SetWallpaper", &path.to_string_lossy().as_ref()).await?;
        }
        Commands::Lock { command } => okshellctl::subcommands::lock::execute(command).await?,
        Commands::Settings { command } => okshellctl::subcommands::settings::execute(command).await?,
    };

    Ok(())
}
