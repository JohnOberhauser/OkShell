use clap::{
    builder::styling::{AnsiColor, Effects, Styles},
};

pub fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Blue.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Magenta.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}