use okshell_config::schema::themes::Themes;
use crate::matugen::json_struct::{MatugenTheme, OkShell};
use crate::static_themes::bloodrust::blood_rust;
use crate::static_themes::catppuccin_frappe::catppuccin_frappe;
use crate::static_themes::catppuccin_latte::catppuccin_latte;
use crate::static_themes::catppuccin_macchiato::catppuccin_macchiato;
use crate::static_themes::catppuccin_mocha::catppuccin_mocha;
use crate::static_themes::desert_power::desert_power;
use crate::static_themes::dracula::dracula;
use crate::static_themes::everforest_dark_hard::everforest_dark_hard;
use crate::static_themes::everforest_dark_medium::everforest_dark_medium;
use crate::static_themes::everforest_dark_soft::everforest_dark_soft;
use crate::static_themes::everforest_light_hard::everforest_light_hard;
use crate::static_themes::everforest_light_medium::everforest_light_medium;
use crate::static_themes::everforest_light_soft::everforest_light_soft;
use crate::static_themes::gruvbox_dark_hard::gruvbox_dark_hard;
use crate::static_themes::gruvbox_dark_medium::gruvbox_dark_medium;
use crate::static_themes::gruvbox_dark_soft::gruvbox_dark_soft;
use crate::static_themes::gruvbox_light_hard::gruvbox_light_hard;
use crate::static_themes::gruvbox_light_medium::gruvbox_light_medium;
use crate::static_themes::gruvbox_light_soft::gruvbox_light_soft;
use crate::static_themes::kanagawa_dragon::kanagawa_dragon;
use crate::static_themes::kanagawa_lotus::kanagawa_lotus;
use crate::static_themes::kanagawa_wave::kanagawa_wave;
use crate::static_themes::nord_dark::nord_dark;
use crate::static_themes::nord_light::nord_light;
use crate::static_themes::one_dark::one_dark;
use crate::static_themes::rose_pine::rose_pine;
use crate::static_themes::rose_pine_dawn::rose_pine_dawn;
use crate::static_themes::rose_pine_moon::rose_pine_moon;
use crate::static_themes::solarized_dark::solarized_dark;
use crate::static_themes::solarized_light::solarized_light;
use crate::static_themes::tokyo_night::tokyo_night;
use crate::static_themes::tokyo_night_light::tokyo_night_light;
use crate::static_themes::tokyo_night_storm::tokyo_night_storm;
use crate::static_themes::varda::varda;

pub fn static_theme(
    theme: &Themes,
    okshell: Option<OkShell>,
) -> Option<MatugenTheme> {
    let okshell = okshell.unwrap_or_default();
    match theme {
        Themes::Default | Themes::Wallpaper => None,
        Themes::BloodRust => Some(blood_rust(okshell)),
        Themes::CatppuccinFrappe => Some(catppuccin_frappe(okshell)),
        Themes::CatppuccinLatte => Some(catppuccin_latte(okshell)),
        Themes::CatppuccinMacchiato => Some(catppuccin_macchiato(okshell)),
        Themes::CatppuccinMocha => Some(catppuccin_mocha(okshell)),
        Themes::DesertPower => Some(desert_power(okshell)),
        Themes::Dracula => Some(dracula(okshell)),
        Themes::EverforestDarkHard => Some(everforest_dark_hard(okshell)),
        Themes::EverforestDarkMedium => Some(everforest_dark_medium(okshell)),
        Themes::EverforestDarkSoft => Some(everforest_dark_soft(okshell)),
        Themes::EverforestLightHard => Some(everforest_light_hard(okshell)),
        Themes::EverforestLightMedium => Some(everforest_light_medium(okshell)),
        Themes::EverforestLightSoft => Some(everforest_light_soft(okshell)),
        Themes::GruvboxDarkHard => Some(gruvbox_dark_hard(okshell)),
        Themes::GruvboxDarkMedium => Some(gruvbox_dark_medium(okshell)),
        Themes::GruvboxDarkSoft => Some(gruvbox_dark_soft(okshell)),
        Themes::GruvboxLightHard => Some(gruvbox_light_hard(okshell)),
        Themes::GruvboxLightMedium => Some(gruvbox_light_medium(okshell)),
        Themes::GruvboxLightSoft => Some(gruvbox_light_soft(okshell)),
        Themes::KanagawaDragon => Some(kanagawa_dragon(okshell)),
        Themes::KanagawaLotus => Some(kanagawa_lotus(okshell)),
        Themes::KanagawaWave => Some(kanagawa_wave(okshell)),
        Themes::NordDark => Some(nord_dark(okshell)),
        Themes::NordLight => Some(nord_light(okshell)),
        Themes::OneDark => Some(one_dark(okshell)),
        Themes::RosePine => Some(rose_pine(okshell)),
        Themes::RosePineDawn => Some(rose_pine_dawn(okshell)),
        Themes::RosePineMoon => Some(rose_pine_moon(okshell)),
        Themes::SolarizedDark => Some(solarized_dark(okshell)),
        Themes::SolarizedLight => Some(solarized_light(okshell)),
        Themes::TokyoNight => Some(tokyo_night(okshell)),
        Themes::TokyoNightStorm => Some(tokyo_night_storm(okshell)),
        Themes::TokyoNightLight => Some(tokyo_night_light(okshell)),
        Themes::Varda => Some(varda(okshell)),
    }
}