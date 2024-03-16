use anyhow::{bail, Result};
use config::{Config, FileFormat};
use dirs::home_dir;
use indicatif::ProgressStyle;
use num_format::Locale;
use rustyline::{ColorMode, EditMode};
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::{debug, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub(crate) static DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/rsql.toml"));

/// A builder for creating a [Configuration] instance.
#[derive(Clone, Debug, Default)]
pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    pub fn new<S: Into<String>>(program_name: S, version: S) -> Self {
        let mut configuration = Configuration::default();
        let program_name = program_name.into();
        let version = version.into();

        configuration.program_name = program_name;
        configuration.version = version;

        Self { configuration }
    }

    /// Initialize configuration from the configuration file.  The configuration file is located
    /// in the user's home directory in a hidden directory named after the program name
    /// (e.g. `.rsql`) or in the current working directory if the home directory is not
    /// available. The configuration file is named after the program name with a `.toml` extension
    /// (e.g. `.rsql.toml`).
    ///
    /// If the configuration file does not exist, it is created with the default configuration.
    pub fn with_config(self) -> Self {
        let home_dir = home_dir().unwrap_or_else(|| env::current_dir().unwrap_or_default());
        let config_dir = home_dir.join(format!(".{}", &self.configuration.program_name));
        self.with_config_dir(config_dir)
    }

    /// Initialize configuration from the configuration file.  The configuration file is located
    /// in the user's home directory in a hidden directory named after the program name
    /// (e.g. `.rsql`) or in the current working directory if the home directory is not
    /// available. The configuration file is named after the program name with a `.toml` extension
    /// (e.g. `.rsql.toml`).
    ///
    /// If the configuration file does not exist, it is created with the default configuration.
    pub fn with_config_dir<P: Into<PathBuf>>(mut self, config_dir: P) -> Self {
        let config_dir = config_dir.into();
        self.configuration.config_dir = Some(config_dir.clone());
        let config_file =
            ConfigFile::new(&self.configuration.program_name, &config_dir).expect("config file");
        config_file
            .load_configuration(&mut self.configuration)
            .expect("load configuration");
        self
    }

    /// Set the bail on error to use.
    #[allow(dead_code)]
    pub fn with_bail_on_error(mut self, bail_on_error: bool) -> Self {
        self.configuration.bail_on_error = bail_on_error;
        self
    }

    /// Set the log level to use.
    #[allow(dead_code)]
    pub fn with_log_level(mut self, log_level: LevelFilter) -> Self {
        self.configuration.log_level = log_level;
        self
    }

    /// Set the log directory to use.
    #[allow(dead_code)]
    pub fn with_log_dir<P: Into<PathBuf>>(mut self, log_dir: P) -> Self {
        self.configuration.log_dir = Some(log_dir.into());
        self
    }

    /// Set the log rotation to use.
    #[allow(dead_code)]
    pub fn with_log_rotation(mut self, log_rotation: Rotation) -> Self {
        self.configuration.log_rotation = log_rotation;
        self
    }

    /// Set the locale to use.
    #[allow(dead_code)]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.configuration.locale = locale;
        self
    }

    /// Set the color mode to use.
    #[allow(dead_code)]
    pub fn with_color_mode(mut self, color_mode: ColorMode) -> Self {
        self.configuration.color_mode = color_mode;
        self
    }

    /// Set the edit mode to use.
    #[allow(dead_code)]
    pub fn with_edit_mode(mut self, edit_mode: EditMode) -> Self {
        self.configuration.edit_mode = edit_mode;
        self
    }

    /// Set the history to use.
    #[allow(dead_code)]
    pub fn with_history(mut self, history: bool) -> Self {
        self.configuration.history = history;
        self
    }

    /// Set the history file to use.
    #[allow(dead_code)]
    pub fn with_history_file<P: Into<PathBuf>>(mut self, history_file: P) -> Self {
        self.configuration.history_file = Some(history_file.into());
        self
    }

    /// Set the history limit to use.
    #[allow(dead_code)]
    pub fn with_history_limit(mut self, history_limit: usize) -> Self {
        self.configuration.history_limit = history_limit;
        self
    }

    /// Set the history ignore dups to use.
    #[allow(dead_code)]
    pub fn with_history_ignore_dups(mut self, history_ignore_dups: bool) -> Self {
        self.configuration.history_ignore_dups = history_ignore_dups;
        self
    }

    /// Set the theme to use.
    #[allow(dead_code)]
    pub fn with_theme<S: Into<String>>(mut self, theme: S) -> Self {
        self.configuration.theme = theme.into();
        self
    }

    /// Set the results format to use.
    #[allow(dead_code)]
    pub fn with_results_format<S: Into<String>>(mut self, results_format: S) -> Self {
        self.configuration.results_format = results_format.into();
        self
    }

    /// Set the results header to use.
    #[allow(dead_code)]
    pub fn with_results_header(mut self, results_header: bool) -> Self {
        self.configuration.results_header = results_header;
        self
    }

    /// Set the results footer to use.
    #[allow(dead_code)]
    pub fn with_results_footer(mut self, results_footer: bool) -> Self {
        self.configuration.results_footer = results_footer;
        self
    }

    /// Set the results timer to use.
    #[allow(dead_code)]
    pub fn with_results_timer(mut self, results_timer: bool) -> Self {
        self.configuration.results_timer = results_timer;
        self
    }

    /// Build a [Configuration] instance.
    pub fn build(self) -> Configuration {
        let configuration = &self.configuration;
        let log_level = configuration.log_level;
        let registry = tracing_subscriber::registry();
        let progress_style =
            ProgressStyle::with_template("{span_child_prefix}{spinner} {span_name}")
                .expect("progress style");

        if log_level != LevelFilter::OFF {
            let log_dir = configuration.log_dir.clone().unwrap_or_default();
            let log_rotation = configuration.log_rotation.clone();
            let level = log_level.into_level().expect("log level");
            let file_appender = RollingFileAppender::builder()
                .rotation(log_rotation)
                .filename_prefix(&configuration.program_name)
                .build(log_dir)
                .expect("log file appender")
                .with_max_level(level);
            let indicatif_layer = IndicatifLayer::new().with_progress_style(progress_style);

            registry
                .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
                .with(indicatif_layer)
                .init();
        } else {
            let indicatif_layer = IndicatifLayer::new().with_progress_style(progress_style);

            registry.with(indicatif_layer).init();
        }

        self.configuration
    }
}

/// The configuration for the application.
#[derive(Clone, Debug)]
pub struct Configuration {
    pub program_name: String,
    pub version: String,
    pub config_dir: Option<PathBuf>,
    pub bail_on_error: bool,
    pub log_level: LevelFilter,
    pub log_dir: Option<PathBuf>,
    pub log_rotation: Rotation,
    pub locale: Locale,
    pub color_mode: ColorMode,
    pub edit_mode: EditMode,
    pub history: bool,
    pub history_file: Option<PathBuf>,
    pub history_limit: usize,
    pub history_ignore_dups: bool,
    pub theme: String,
    pub results_format: String,
    pub results_header: bool,
    pub results_footer: bool,
    pub results_timer: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            program_name: String::new(),
            version: String::new(),
            config_dir: None,
            bail_on_error: false,
            log_level: LevelFilter::OFF,
            log_dir: None,
            log_rotation: Rotation::DAILY,
            locale: Locale::en,
            color_mode: ColorMode::Forced,
            edit_mode: EditMode::Emacs,
            history: false,
            history_file: None,
            history_limit: 1000,
            history_ignore_dups: true,
            theme: "Solarized (dark)".to_string(),
            results_format: "unicode".to_string(),
            results_header: true,
            results_footer: true,
            results_timer: true,
        }
    }
}

/// The configuration file for the application.
#[derive(Clone, Debug)]
struct ConfigFile {
    program_name: String,
    config_dir: PathBuf,
    config: Config,
}

impl ConfigFile {
    fn new<S: Into<String>, P: Into<PathBuf>>(
        program_name: S,
        config_dir: P,
    ) -> Result<ConfigFile> {
        let program_name = program_name.into();
        let config_dir = config_dir.into();

        create_dir_all(&config_dir)?;
        let configuration_file = config_dir.join(format!("{program_name}.toml"));

        // Create the configuration file if it does not exist
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&configuration_file)
        {
            file.write_all(DEFAULT_CONFIG.as_bytes())?;
        }

        let conf_file = configuration_file.to_str().expect("config file");
        debug!("Configuration file: {conf_file}");

        let prefix = program_name.to_uppercase().replace('-', "_");
        debug!("Configuration environment prefix: {prefix}");

        let config = Config::builder()
            .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(config::File::new(conf_file, FileFormat::Toml))
            .add_source(config::Environment::with_prefix(prefix.as_str()).separator("_"))
            .build()?;

        Ok(Self {
            program_name,
            config_dir,
            config,
        })
    }

    fn load_configuration(&self, configuration: &mut Configuration) -> Result<()> {
        let config = &self.config;
        let config_dir = &self.config_dir;

        if let Ok(bail_on_error) = config.get::<bool>("general.bail_on_error") {
            configuration.bail_on_error = bail_on_error;
        }

        if let Ok(log_level) = config.get::<String>("log.level") {
            configuration.log_level = LevelFilter::from_str(log_level.as_str())?;
        }

        configuration.log_dir = Some(config_dir.join("logs"));

        configuration.log_rotation = match config.get::<String>("log.rotation")?.as_str() {
            "minutely" => Rotation::MINUTELY,
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            "never" => Rotation::NEVER,
            rotation => bail!("Invalid log.rotation: {rotation}"),
        };

        configuration.locale = get_locale(config);

        configuration.edit_mode = match config.get::<String>("shell.edit_mode")?.as_str() {
            "emacs" => EditMode::Emacs,
            "vi" => EditMode::Vi,
            mode => bail!("Invalid shell.edit_mode: {mode}"),
        };

        if let Ok(history) = config.get("shell.history.enabled") {
            configuration.history = history;
        }
        let history_file = config_dir.join(format!("{}.history", &self.program_name));
        configuration.history_file = Some(history_file);
        if let Ok(history_limit) = config.get("shell.history.limit") {
            configuration.history_limit = history_limit;
        }
        if let Ok(history_ignore_dups) = config.get("shell.history.ignore_dups") {
            configuration.history_ignore_dups = history_ignore_dups;
        }

        configuration.theme = theme(config)?;

        if let Ok(results_format) = config.get::<String>("results.format") {
            configuration.results_format = results_format;
        }
        if let Ok(results_header) = config.get::<bool>("results.header") {
            configuration.results_header = results_header;
        }
        if let Ok(results_footer) = config.get::<bool>("results.footer") {
            configuration.results_footer = results_footer;
        }
        if let Ok(results_timer) = config.get::<bool>("results.timer") {
            configuration.results_timer = results_timer;
        }

        Ok(())
    }
}

fn get_locale(config: &Config) -> Locale {
    let default_locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    let locale = config.get("global.locale").unwrap_or(default_locale);
    let parts: Vec<&str> = locale
        .split(|c: char| !c.is_alphanumeric())
        .filter(|&s| !s.is_empty())
        .collect();

    for i in (0..parts.len()).rev() {
        let locale = parts[0..=i].join("-");
        if let Ok(locale) = Locale::from_str(locale.as_str()) {
            return locale;
        }
    }

    warn!("Invalid locale: {locale}; defaulting to \"en\"");
    Locale::en
}

fn theme(config: &Config) -> Result<String> {
    if let Ok(theme) = config.get("shell.theme") {
        return Ok(theme);
    }

    let timeout = Duration::from_millis(20);
    let mode = match termbg::theme(timeout) {
        Ok(termbg::Theme::Dark) => dark_light::Mode::Dark,
        Ok(termbg::Theme::Light) => dark_light::Mode::Light,
        Err(_) => dark_light::detect(),
    };

    let config_key = match mode {
        dark_light::Mode::Dark | dark_light::Mode::Default => "shell.theme.dark",
        dark_light::Mode::Light => "shell.theme.light",
    };

    Ok(config.get(config_key)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_configuration_builder() {
        let program_name = "test";
        let version = "1.2.3";
        let bail_on_error = true;
        let log_level = LevelFilter::OFF;
        let log_dir = ".rsql/logs";
        let log_rotation = Rotation::MINUTELY;
        let locale = Locale::es;
        let color_mode = ColorMode::Disabled;
        let edit_mode = EditMode::Vi;
        let history = true;
        let history_file = ".rsql/history.txt";
        let history_limit = 42;
        let history_ignore_dups = false;
        let theme = "Solarized (light)";
        let results_format = "unicode".to_string();
        let results_header = false;
        let results_footer = false;
        let results_timer = false;

        let configuration = ConfigurationBuilder::new(program_name, version)
            .with_bail_on_error(bail_on_error)
            .with_log_level(log_level)
            .with_log_dir(log_dir)
            .with_log_rotation(log_rotation.clone())
            .with_locale(locale)
            .with_color_mode(color_mode)
            .with_edit_mode(edit_mode)
            .with_history(history)
            .with_history_file(history_file)
            .with_history_limit(history_limit)
            .with_history_ignore_dups(history_ignore_dups)
            .with_theme(theme)
            .with_results_format(results_format.clone())
            .with_results_header(results_header)
            .with_results_footer(results_footer)
            .with_results_timer(results_timer)
            .build();

        assert_eq!(configuration.program_name, program_name);
        assert_eq!(configuration.version, version);
        assert_eq!(configuration.bail_on_error, bail_on_error);
        assert_eq!(configuration.log_level, log_level);
        assert_eq!(configuration.log_dir.unwrap().to_string_lossy(), log_dir);
        assert_eq!(configuration.log_rotation, log_rotation);
        assert_eq!(configuration.locale, locale);
        assert_eq!(configuration.color_mode, color_mode);
        assert_eq!(configuration.edit_mode, edit_mode);
        assert_eq!(configuration.history, history);
        assert_eq!(
            configuration.history_file.unwrap().to_string_lossy(),
            history_file
        );
        assert_eq!(configuration.history_limit, history_limit);
        assert_eq!(configuration.history_ignore_dups, history_ignore_dups);
        assert_eq!(configuration.theme, theme);
        assert_eq!(configuration.results_format, results_format);
        assert_eq!(configuration.results_header, results_header);
        assert_eq!(configuration.results_footer, results_footer);
        assert_eq!(configuration.results_timer, results_timer);
    }

    #[test]
    fn test_default_configuration() {
        let configuration = Configuration::default();
        assert!(configuration.program_name.is_empty());
        assert!(configuration.version.is_empty());
        assert_eq!(configuration.config_dir, None);
        assert_eq!(configuration.bail_on_error, false);
        assert_eq!(configuration.log_level, LevelFilter::OFF);
        assert_eq!(configuration.log_dir, None);
        assert_eq!(configuration.log_rotation, Rotation::DAILY);
        assert_eq!(configuration.locale, Locale::en);
        assert_eq!(configuration.color_mode, ColorMode::Forced);
        assert_eq!(configuration.edit_mode, EditMode::Emacs);
        assert_eq!(configuration.history, false);
        assert_eq!(configuration.history_file, None);
        assert_eq!(configuration.history_limit, 1000);
        assert_eq!(configuration.history_ignore_dups, true);
        assert_eq!(configuration.theme, "Solarized (dark)");
        assert_eq!(configuration.results_format, "unicode".to_string());
        assert_eq!(configuration.results_header, true);
        assert_eq!(configuration.results_footer, true);
        assert_eq!(configuration.results_timer, true);
    }

    #[test]
    fn test_get_locale_language() -> Result<()> {
        let prefix = "LOCALE_LANGUAGE_TEST";
        env::set_var(format!("{prefix}_GLOBAL_LOCALE"), "de-US.foo");
        let config = Config::builder()
            .add_source(config::Environment::with_prefix(prefix).separator("_"))
            .build()?;
        let locale = get_locale(&config);
        assert_eq!(locale, Locale::de);
        Ok(())
    }

    #[test]
    fn test_get_locale_language_and_country() -> Result<()> {
        let prefix = "LOCALE_LANGUAGE_AND_COUNTRY_TEST";
        env::set_var(format!("{prefix}_GLOBAL_LOCALE"), "en_GB.foo");
        let config = Config::builder()
            .add_source(config::Environment::with_prefix(prefix).separator("_"))
            .build()?;
        let locale = get_locale(&config);
        assert_eq!(locale, Locale::en_GB);
        Ok(())
    }

    #[test]
    fn test_get_locale_default() -> Result<()> {
        let prefix = "LOCALE_DEFAULT_TEST";
        env::set_var(format!("{prefix}_GLOBAL_LOCALE"), "foo");
        let config = Config::builder()
            .add_source(config::Environment::with_prefix(prefix).separator("_"))
            .build()?;
        let locale = get_locale(&config);
        assert_eq!(locale, Locale::en);
        Ok(())
    }
}
