extern crate chrono;
extern crate itertools;
extern crate log;
extern crate thiserror;

mod error;
mod pretty;

use chrono::Local;
use error::*;
use itertools::Itertools;
use log::LevelFilter;
use std::{
    collections::HashMap,
    fmt::Arguments,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct SkuldLogger {
    level: LevelFilter,
    modules: HashMap<String, LevelFilter>,
    fmt: &'static str,
    file: Arc<Mutex<File>>,
}

impl SkuldLogger {
    pub fn new(path: PathBuf) -> Result<Self, CreateLoggerError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;

        Ok(SkuldLogger {
            level: LevelFilter::Info,
            modules: HashMap::new(),
            file: Arc::new(Mutex::new(file)),
            fmt: "%Y-%m-%d %l:%M:%S%.3f %p",
        })
    }

    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    pub fn with_module(mut self, module: impl Into<String>, level: LevelFilter) -> Self {
        self.modules.insert(module.into(), level);
        self
    }

    pub fn max_level(&self) -> LevelFilter {
        self.modules
            .values()
            .copied()
            .max()
            .unwrap_or(self.level)
            .max(self.level)
    }

    pub fn date_fmt(mut self, date_fmt: &'static str) -> Self {
        self.fmt = date_fmt;
        self
    }

    pub fn init(self) -> Result<(), CreateLoggerError> {
        log::set_max_level(self.max_level());
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }

    fn write(&self, message: String) -> Result<(), WriteFileError<'_>> {
        let mut file = self.file.lock()?;
        file.write_all(message.as_bytes())?;

        Ok(())
    }

    fn flush(&self) -> Result<(), WriteFileError<'_>> {
        let mut file = self.file.lock()?;
        file.flush()?;

        Ok(())
    }

    fn multiline_message(args: &Arguments) -> String {
        let msg = args.to_string().trim().to_string();

        if msg.contains("\n") {
            msg.split("\n").map(|s| format!("\t{s}")).join("\n")
        } else {
            msg
        }
    }
}

impl log::Log for SkuldLogger {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        meta.level()
            <= *self
                .modules
                .iter()
                .find(|(name, _level)| meta.target().starts_with(*name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.level)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = Local::now().format(self.fmt).to_string().trim().to_string();
        let level = record.level();
        let module = record.target();
        let message = SkuldLogger::multiline_message(record.args());

        let formatted = {
            let message = pretty::light(&message);
            let level = pretty::level(level);
            let module = pretty::bold(module);

            format!("{time} {level} [{module}] {message}\n")
        };

        let unformatted = format!("{time} {level} [{module}] {message}\n");

        print!("{}", formatted);
        self.write(unformatted).unwrap();
    }

    fn flush(&self) {
        self.flush().unwrap();
    }
}

pub mod prelude {
    pub use super::error::*;
    pub use super::SkuldLogger;
}
