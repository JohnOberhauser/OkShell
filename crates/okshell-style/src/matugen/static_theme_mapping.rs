use okshell_config::schema::themes::Themes;
use crate::matugen::json_struct::MatugenTheme;
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
use crate::static_themes::nord_dark::nord_dark;
use crate::static_themes::nord_light::nord_light;
use crate::static_themes::rose_pine::rose_pine;
use crate::static_themes::rose_pine_dawn::rose_pine_dawn;
use crate::static_themes::rose_pine_moon::rose_pine_moon;
use crate::static_themes::tokyo_night::tokyo_night;
use crate::static_themes::tokyo_night_light::tokyo_night_light;
use crate::static_themes::tokyo_night_storm::tokyo_night_storm;
use crate::static_themes::varda::varda;

pub fn static_theme(theme: &Themes) -> Option<MatugenTheme> {
    match theme {
        Themes::Default | Themes::Matugen => None,
        Themes::BloodRust => Some(blood_rust()),
        Themes::CatppuccinFrappe => Some(catppuccin_frappe()),
        Themes::CatppuccinLatte => Some(catppuccin_latte()),
        Themes::CatppuccinMacchiato => Some(catppuccin_macchiato()),
        Themes::CatppuccinMocha => Some(catppuccin_mocha()),
        Themes::DesertPower => Some(desert_power()),
        Themes::Dracula => Some(dracula()),
        Themes::EverforestDarkHard => Some(everforest_dark_hard()),
        Themes::EverforestDarkMedium => Some(everforest_dark_medium()),
        Themes::EverforestDarkSoft => Some(everforest_dark_soft()),
        Themes::EverforestLightHard => Some(everforest_light_hard()),
        Themes::EverforestLightMedium => Some(everforest_light_medium()),
        Themes::EverforestLightSoft => Some(everforest_light_soft()),
        Themes::GruvboxDarkHard => Some(gruvbox_dark_hard()),
        Themes::GruvboxDarkMedium => Some(gruvbox_dark_medium()),
        Themes::GruvboxDarkSoft => Some(gruvbox_dark_soft()),
        Themes::GruvboxLightHard => Some(gruvbox_light_hard()),
        Themes::GruvboxLightMedium => Some(gruvbox_light_medium()),
        Themes::GruvboxLightSoft => Some(gruvbox_light_soft()),
        Themes::NordDark => Some(nord_dark()),
        Themes::NordLight => Some(nord_light()),
        Themes::RosePine => Some(rose_pine()),
        Themes::RosePineDawn => Some(rose_pine_dawn()),
        Themes::RosePineMoon => Some(rose_pine_moon()),
        Themes::TokyoNight => Some(tokyo_night()),
        Themes::TokyoNightStorm => Some(tokyo_night_storm()),
        Themes::TokyoNightLight => Some(tokyo_night_light()),
        Themes::Varda => Some(varda()),
    }
}