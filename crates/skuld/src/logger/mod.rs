extern crate chrono;
extern crate itertools;
extern crate log;
extern crate thiserror;

mod error;
mod pretty;

use error::*;
use itertools::Itertools;
use log::LevelFilter;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct SkuldLogger {
    level: LevelFilter,
    modules: HashMap<String, LevelFilter>,
    file: Arc<Mutex<File>>,
    date_fmt: &'static str,
}

impl SkuldLogger {
    pub fn new(path: PathBuf) -> Result<Self, CreateLoggerError> {
        let file = File::open(path.clone()).or_else(|_| File::create_new(path))?;

        Ok(SkuldLogger {
            level: LevelFilter::Info,
            modules: HashMap::new(),
            file: Arc::new(Mutex::new(file)),
            date_fmt: "%Y-%m-%d %l:%M:%S%.3f %p",
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
        self.date_fmt = date_fmt;
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
}

impl log::Log for SkuldLogger {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        meta.level() <= self.max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut message = record.args().to_string();

        if message.contains("\n") {
            message = message.split("\n").map(|s| format!("\t{s}")).join("\n");
        }

        let formatted = {
            let time = chrono::Local::now().format(self.date_fmt).to_string();
            let level = pretty::level(record.level());
            let module = pretty::bold(record.target().to_string());
            let message = pretty::light(&message);

            format!("{time} {level} [{module}] {message}\n")
        };

        let unformatted = {
            let time = chrono::Local::now().format(self.date_fmt).to_string();
            let level = record.level();
            let module = record.target();
            let message = &message;

            format!("{time} {level} [{module}] {message}\n")
        };

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
