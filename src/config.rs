use std::{io::Write, path::Path};
use std::str::FromStr;
use std::process;
use toml::Value;
use log::{warn, error};
use shellexpand;

#[derive(Debug, Default)]
pub struct Config {
    pub arg_search: bool,
    pub arg_terminal_input: bool,
    pub arg_copy_input: bool,
    pub arg_entries: Vec<String>,
    pub arg_entries_delete: Vec<String>,
    
    pub path: String,
    pub wl_seat: String,
    pub primary_clip: bool,
    pub silent: bool,
    
    tbl: toml::Table,
}

impl Config {
    pub fn init(&mut self, config_file: &str, config_dir: &str) -> std::io::Result<()> {
        let config_dir_path = Path::new(config_dir);
        if !config_dir_path.exists() {
            warn!("clippyman config folder was not found, Creating folders at {}!", config_dir);
            std::fs::create_dir_all(config_dir_path)?;
        }

        let config_file_path = Path::new(config_file);
        if !config_file_path.exists() {
            warn!("config file {} not found, generating new one", config_file);
            self.generate_config(config_file)?;
        }
        
        Ok(())
    }

    pub fn load_config_file(&mut self, filename: &str) {
        let contents = match std::fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read config file '{}': {}", filename, e);
                process::exit(1);
            }
        };

        self.tbl = match contents.parse::<toml::Table>() {
            Ok(t) => t,
            Err(e) => {
                error!("Parsing config file '{}' failed:\n{}", filename, e);
                process::exit(1);
            }
        };

        self.path = self.get_value("config.path", "~/.cache/crubclip/history.json".to_owned());
        self.wl_seat = self.get_value("config.wl-seat", "".to_owned());
        self.primary_clip = self.get_value("config.primary", false);
        self.silent = self.get_value("config.silent", false);
    }

    pub fn generate_config(&self, filename: &str) -> std::io::Result<()> {
        let path = Path::new(filename);
        if path.exists() {
            if !self.ask_user_yn(false, &format!("WARNING: config file '{}' already exists. Do you want to overwrite it?\n", filename)) {
                process::exit(1);
            }
        }

        std::fs::write(filename, AUTOCONFIG)?;
        Ok(())
    }

    fn ask_user_yn(&self, default: bool, prompt: &str) -> bool {
        print!("{} [y/n] (default: {}) ", prompt, if default { "yes" } else { "no" });
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => default,
        }
    }

    fn get_value<T: FromStr>(&self, path: &str, fallback: T) -> T 
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let mut current = &Value::Table(self.tbl.clone());
        
        for part in path.split('.') {
            if let Value::Table(table) = current {
                current = table.get(part).unwrap();//_or(&Value::String(String::new()));
            } else {
                return fallback;
            }
        }

        match current {
            Value::String(s) => {
                let expanded = shellexpand::full(s).unwrap_or_else(|_| s.into());
                expanded.parse().unwrap_or(fallback)
            },
            Value::Boolean(b) => T::from_str(&b.to_string()).unwrap_or(fallback),
            Value::Integer(i) => T::from_str(&i.to_string()).unwrap_or(fallback),
            Value::Float(f) => T::from_str(&f.to_string()).unwrap_or(fallback),
            _ => fallback,
        }
    }
}

const AUTOCONFIG: &str = r#"[config]
# Path to where we store the clipboard history
path = "~/.cache/clipcrab/history.json"

# Use the primary clipboard instead
primary = false

# The seat for using in wayland (i don't know what that is tbh, just leave it empty)
wl-seat = ""

# Print an info message along the search content you selected
silent = false
"#;
