//! Internal logging for the FunnelMob SDK.

use crate::configuration::LogLevel;

/// Internal logger that respects the configured log level.
#[derive(Clone)]
pub struct Logger {
    level: LogLevel,
    prefix: &'static str,
}

impl Logger {
    /// Creates a new logger with the specified log level.
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            prefix: "[FunnelMob]",
        }
    }

    /// Logs an error message.
    pub fn error(&self, message: &str) {
        if self.level >= LogLevel::Error {
            log::error!("{} {}", self.prefix, message);
        }
    }

    /// Logs a warning message.
    pub fn warn(&self, message: &str) {
        if self.level >= LogLevel::Warning {
            log::warn!("{} {}", self.prefix, message);
        }
    }

    /// Logs an info message.
    pub fn info(&self, message: &str) {
        if self.level >= LogLevel::Info {
            log::info!("{} {}", self.prefix, message);
        }
    }

    /// Logs a debug message.
    pub fn debug(&self, message: &str) {
        if self.level >= LogLevel::Debug {
            log::debug!("{} {}", self.prefix, message);
        }
    }

    /// Logs a verbose message.
    pub fn verbose(&self, message: &str) {
        if self.level >= LogLevel::Verbose {
            log::trace!("{} {}", self.prefix, message);
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Debug);
        assert_eq!(logger.prefix, "[FunnelMob]");
    }

    #[test]
    fn test_logger_default() {
        let logger = Logger::default();
        // Default should be None level
        assert_eq!(logger.level, LogLevel::None);
    }

    // Note: Testing actual log output requires a log subscriber setup
    // which is beyond the scope of these unit tests
}
