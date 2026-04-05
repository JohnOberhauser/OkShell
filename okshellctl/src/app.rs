use std::path::PathBuf;
use clap::{
    Parser, Subcommand,
};

use okshell_cli_style;
use crate::subcommands::audio::AudioCommands;
use crate::subcommands::brightness::BrightnessCommands;
use crate::subcommands::lock::LockCommands;
use crate::subcommands::menu::MenuCommands;
use crate::subcommands::settings::SettingsCommands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(styles = okshell_cli_style::get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Kill okshell
    Quit,
    /// Launch the GTK4 inspector.  Useful for finding css node id's and classes.
    Inspect,
    /// Set the current wallpaper
    SetWallpaper {
        /// Path to the image file
        path: PathBuf,
    },
    /// Commands for opening and closing menus
    Menu {
        #[command(subcommand)]
        command: MenuCommands,
    },
    /// Commands for changing audio
    Audio {
        #[command(subcommand)]
        command: AudioCommands,
    },
    /// Commands for changing brightness
    Brightness {
        #[command(subcommand)]
        command: BrightnessCommands,
    },
    /// Commands for locking the session
    Lock {
        #[command(subcommand)]
        command: LockCommands,
    },
    /// Commands for the settings window
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
    }
}