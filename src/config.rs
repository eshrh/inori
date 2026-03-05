use crate::event_handler::Result;
use crate::model::*;
use crate::view::Theme;
use platform_dirs::AppDirs;
use ratatui::style::Style;
use std::error::Error;
use std::fmt;
use std::fs;
use toml::Table;
use toml::Value;
pub mod keybind;
use keybind::{get_message, KeybindMap, KeybindParseError};

pub struct Config {
    pub keybindings: KeybindMap,
    pub theme: Theme,
    pub seek_seconds: i64,
    pub mpd_address: Option<String>,
    pub screens: Vec<Screen>,
    pub nucleo_prefer_prefix: bool,
}

impl Config {
    pub fn default() -> Self {
        Config {
            keybindings: KeybindMap::default(),
            theme: Theme::new(),
            seek_seconds: 5,
            mpd_address: None,
            screens: vec![Screen::Library, Screen::Queue],
            nucleo_prefer_prefix: false,
        }
    }

    pub fn try_read_config(mut self) -> Result<Self> {
        let app_dirs = AppDirs::new(Some("inori"), true);
        let config_file_path =
            app_dirs.map(|d| d.config_dir.join("config.toml"));

        if let Some(Ok(contents)) = config_file_path.map(fs::read_to_string) {
            let toml = contents.parse::<Table>()?; //failed to parse toml
            for (key, value) in toml {
                match (key.as_str(), value) {
                    ("keybindings", Value::Table(t)) => {
                        self.read_keybinds(t)?
                    }
                    ("keybindings", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    ("seek_seconds", Value::Integer(k)) => {
                        if k > 0 {
                            self.seek_seconds = k
                        } else {
                            return Err(Box::new(ConfigError::InvalidValue {
                                key,
                                value: Value::Integer(k),
                            }));
                        }
                    }
                    ("seek_seconds", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    ("theme", Value::Table(t)) => {
                        self.theme = self.theme.apply_theme(t)?
                    }
                    ("theme", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    ("dvorak_keybindings", Value::Boolean(true)) => {
                        self.keybindings = self.keybindings.with_dvorak_style();
                    }
                    ("dvorak_keybindings", Value::Boolean(false)) => {}
                    ("qwerty_keybindings", Value::Boolean(true)) => {
                        self.keybindings = self.keybindings.with_qwerty_style();
                    }
                    ("qwerty_keybindings", Value::Boolean(false)) => {}
                    ("mpd_address", Value::String(addr)) => {
                        self.mpd_address = Some(addr);
                    }
                    ("mpd_address", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    ("screens", Value::Array(screens)) => {
                        self.screens = screens
                            .iter()
                            .map(|v| match v {
                                Value::String(s) => Screen::parse(s)
                                    .ok_or_else(|| {
                                        Box::new(ConfigError::ScreenParse(
                                            s.clone(),
                                        ))
                                            as Box<dyn Error>
                                    }),
                                x => Err(Box::new(
                                    ConfigError::WrongKeyValueType {
                                        key: key.to_owned(),
                                        value: x.to_owned(),
                                    },
                                )
                                    as Box<dyn Error>),
                            })
                            .collect::<Result<Vec<Screen>>>()?;
                    }
                    ("screens", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    ("nucleo_prefer_prefix", Value::Boolean(t)) => {
                        self.nucleo_prefer_prefix = t
                    }
                    ("nucleo_prefer_prefix", other) => {
                        return Err(Box::new(ConfigError::WrongKeyValueType {
                            key,
                            value: other,
                        }));
                    }
                    (unknown, _v) => {
                        return Err(Box::new(ConfigError::UnknownKey(
                            unknown.to_string(),
                        )))
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn read_keybinds(&mut self, t: Table) -> Result<()> {
        for (key, value) in t {
            match (get_message(&key), value) {
                (Some(m), Value::String(s)) => {
                    let keybinds = keybind::parse_keybind(&s).map_err(|e| {
                        Box::new(ConfigError::KeybindParse {
                            command: key.clone(),
                            source: e,
                        }) as Box<dyn Error>
                    })?;
                    self.keybindings.insert(m.clone(), &keybinds);
                }
                (Some(m), Value::Array(a)) => {
                    for v in a {
                        if let Value::String(s) = v {
                            let keybinds =
                                keybind::parse_keybind(&s).map_err(|e| {
                                    Box::new(ConfigError::KeybindParse {
                                        command: key.clone(),
                                        source: e,
                                    })
                                        as Box<dyn Error>
                                })?;
                            self.keybindings.insert(m.clone(), &keybinds);
                        } else {
                            return Err(Box::new(
                                ConfigError::WrongKeyValueType {
                                    key,
                                    value: v,
                                },
                            ));
                        }
                    }
                }
                (Some(_m), other) => {
                    return Err(Box::new(ConfigError::WrongKeyValueType {
                        key,
                        value: other,
                    }))
                }
                (None, _) => {
                    return Err(Box::new(ConfigError::MissingMessage(key)))
                }
            }
        }
        Ok(())
    }
}

//does not log or throw. it simply ignores 'bad' mods
fn join_modifier_array(modifiers: &[Value]) -> String {
    let modifier_strings: Vec<String> = modifiers
        .iter()
        .filter_map(|m| m.as_str().map(|s| s.to_string()))
        .collect();

    let mod_string: String = modifier_strings.join("|");
    mod_string + "|"
}

pub fn deserialize_style(mut t: Table) -> Result<Style> {
    if !t.contains_key("add_modifier") {
        t.insert("add_modifier".into(), Value::String("".into()));
    }
    if !t.contains_key("sub_modifier") {
        t.insert("sub_modifier".into(), Value::String("".into()));
    }
    match t.get("add_modifier") {
        Some(Value::Array(a)) => {
            t.insert(
                "add_modifier".into(),
                Value::String(join_modifier_array(a)),
            );
        }
        Some(Value::String(_)) => {}
        Some(v) => {
            return Err(Box::new(ConfigError::WrongKeyValueType {
                key: "add_modifier".into(),
                value: v.clone(),
            }))
        }
        None => {}
    }
    match t.get("sub_modifier") {
        Some(Value::Array(a)) => {
            t.insert(
                "sub_modifier".into(),
                Value::String(join_modifier_array(a)),
            );
        }
        Some(Value::String(_)) => {}
        Some(v) => {
            return Err(Box::new(ConfigError::WrongKeyValueType {
                key: "sub_modifier".into(),
                value: v.clone(),
            }))
        }
        None => {}
    }
    Ok(t.try_into()?)
}

#[derive(Debug)]
pub enum ConfigError {
    MissingMessage(String),
    //UnknownModifier(String),
    UnknownThemeOption(String),
    UnknownKey(String),
    WrongKeyValueType {
        key: String,
        value: Value,
    },
    InvalidValue {
        key: String,
        value: Value,
    },
    KeybindParse {
        command: String,
        source: KeybindParseError,
    },
    ScreenParse(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingMessage(s) => {
                write!(f, "message {} does not exist", s)
            }
            //ConfigError::UnknownModifier(s) => write!(f, "Error while parsing theme modifier array: unknown modifier: {}", s),
            ConfigError::UnknownThemeOption(s) => {
                write!(f, "theme option {} not found", s)
            }
            ConfigError::UnknownKey(s) => {
                write!(f, "unknown config key {}", s)
            }
            ConfigError::WrongKeyValueType { key, value } => {
                write!(f, "config key {} has wrong type: {}", key, value)
            }
            ConfigError::InvalidValue { key, value } => {
                write!(f, "config key {} has invalid value: {}", key, value)
            }
            ConfigError::KeybindParse { command, source } => {
                write!(f, "failed to parse keybind {}: {}", command, source)
            }
            ConfigError::ScreenParse(s) => {
                write!(f, "unknown screen: {}", s)
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::KeybindParse { source, .. } => Some(source),
            _ => None,
        }
    }
}
