/// Declares a widget enum once, generating the enum itself, `display_name()`,
/// `all_defaults()`, and a lenient `Deserialize`.
///
/// Every variant accepts all of these forms, whether or not it carries config:
///   - `- Clock`                      bare name -> default config
///   - `- Clock:`                     null -> default config
///   - `- Clock: { ... }`             map form
///   - `- !Clock { ... }`             YAML tag form (what serde_yaml writes)
/// A unit variant ignores any config it's given; a config variant falls back
/// to `Default`. Adding or removing config on a variant never breaks existing
/// YAML. Config types must impl `Default` and use `#[serde(default)]`.
///
/// ```ignore
/// widget_enum! {
///     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Store, JsonSchema)]
///     pub enum BarWidget {
///         Clock => "Clock",
///         Cava(CavaConfig) => "Cava",
///     }
/// }
/// ```
/// Don't derive `Deserialize` yourself — the macro provides it.
macro_rules! widget_enum {
    (@make $name:ident $variant:ident) => { $name::$variant };
    (@make $name:ident $variant:ident $cfg:ty) => { $name::$variant(<$cfg as ::core::default::Default>::default()) };

    (@map_value $name:ident $map:ident $variant:ident) => {{
        $map.next_value::<::serde::de::IgnoredAny>()?;
        $name::$variant
    }};
    (@map_value $name:ident $map:ident $variant:ident $cfg:ty) => {
        $name::$variant($map.next_value::<::core::option::Option<$cfg>>()?.unwrap_or_default())
    };

    (@enum_value $name:ident $v:ident $variant:ident) => {{
        ::serde::de::VariantAccess::unit_variant($v)?;
        $name::$variant
    }};
    (@enum_value $name:ident $v:ident $variant:ident $cfg:ty) => {
        $name::$variant(
            ::serde::de::VariantAccess::newtype_variant::<::core::option::Option<$cfg>>($v)?
                .unwrap_or_default(),
        )
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $variant:ident $( ($cfg:ty) )? => $display:literal ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $variant $( ($cfg) )? ),*
        }

        impl $name {
            pub const VARIANTS: &'static [&'static str] = &[ $( stringify!($variant) ),* ];

            pub fn display_name(&self) -> &'static str {
                match self {
                    $( $name::$variant { .. } => $display, )*
                }
            }

            /// Every variant, with default config where applicable.
            pub fn all_defaults() -> ::std::vec::Vec<$name> {
                ::std::vec![ $( widget_enum!(@make $name $variant $($cfg)?) ),* ]
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> ::core::result::Result<Self, D::Error> {
                use ::serde::de::Error as _;
                struct WidgetVisitor;

                impl<'de> ::serde::de::Visitor<'de> for WidgetVisitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        f.write_str(concat!("a ", stringify!($name), " name, or a map of `Name: { config }`"))
                    }

                    fn visit_str<E: ::serde::de::Error>(self, s: &str) -> ::core::result::Result<$name, E> {
                        match s {
                            $( stringify!($variant) => Ok(widget_enum!(@make $name $variant $($cfg)?)), )*
                            other => Err(E::unknown_variant(other, $name::VARIANTS)),
                        }
                    }

                    fn visit_map<A: ::serde::de::MapAccess<'de>>(self, mut map: A) -> ::core::result::Result<$name, A::Error> {
                        let key: ::std::string::String = map
                            .next_key()?
                            .ok_or_else(|| A::Error::invalid_length(0, &self))?;
                        let widget = match key.as_str() {
                            $( stringify!($variant) => widget_enum!(@map_value $name map $variant $($cfg)?), )*
                            other => return Err(A::Error::unknown_variant(other, $name::VARIANTS)),
                        };
                        if map.next_key::<::serde::de::IgnoredAny>()?.is_some() {
                            return Err(A::Error::custom(concat!(
                                "a ", stringify!($name), " entry must have exactly one key"
                            )));
                        }
                        Ok(widget)
                    }

                    fn visit_enum<A: ::serde::de::EnumAccess<'de>>(self, data: A) -> ::core::result::Result<$name, A::Error> {
                        let (key, v): (::std::string::String, A::Variant) = data.variant()?;
                        Ok(match key.as_str() {
                            $( stringify!($variant) => widget_enum!(@enum_value $name v $variant $($cfg)?), )*
                            other => return Err(A::Error::unknown_variant(other, $name::VARIANTS)),
                        })
                    }
                }

                d.deserialize_any(WidgetVisitor)
            }
        }
    };
}

pub(crate) use widget_enum;
