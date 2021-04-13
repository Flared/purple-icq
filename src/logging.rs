use crate::messages::{FdSender, SystemMessage};
use std::cell::RefCell;
use std::sync::Mutex;

std::thread_local! {
    pub static LOGGER: RefCell<Option<Box<dyn log::Log>>> = RefCell::new(None);
}

lazy_static::lazy_static! {
    static ref PURPLE_BUFFER: Mutex<Vec<(String, log::Level, String)>> = Default::default();
}

static TLS_LOGGER: ThreadLocalLogger = ThreadLocalLogger;

pub struct ThreadLocalLogger;

impl log::Log for ThreadLocalLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        LOGGER.with(|cell| {
            if let Some(ref logger) = cell.borrow().as_ref() {
                logger.log(record);
            }
        })
    }

    fn flush(&self) {
        LOGGER.with(|cell| {
            if let Some(ref logger) = cell.borrow().as_ref() {
                logger.flush()
            }
        })
    }
}

pub struct PurpleDebugLogger;

impl log::Log for PurpleDebugLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        let purple_level = match record.level() {
            log::Level::Error => purple::PurpleDebugLevel::PURPLE_DEBUG_ERROR,
            log::Level::Warn => purple::PurpleDebugLevel::PURPLE_DEBUG_WARNING,
            log::Level::Info => purple::PurpleDebugLevel::PURPLE_DEBUG_INFO,
            _ => purple::PurpleDebugLevel::PURPLE_DEBUG_MISC,
        };

        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };
        let line = format!("[{}] {}\n", target, record.args());
        purple::debug(purple_level, "", &line);
    }

    fn flush(&self) {
        let buffer = {
            match PURPLE_BUFFER.lock() {
                Ok(mut buffer) => buffer.split_off(0),
                Err(_) => return,
            }
        };
        for (target, level, message) in buffer {
            log::log!(target: &target, level, "{}", message);
        }
    }
}

pub struct RemoteLogger(pub FdSender<SystemMessage>);

impl log::Log for RemoteLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        if let Ok(mut buffer) = PURPLE_BUFFER.lock() {
            buffer.push((target.into(), record.level(), record.args().to_string()));
        }
    }

    fn flush(&self) {
        self.0.clone().try_send(SystemMessage::FlushLogs);
    }
}

pub fn init(level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_logger(&TLS_LOGGER).map(|()| log::set_max_level(level))
}

pub fn set_thread_logger<T>(logger: T)
where
    T: log::Log + 'static,
{
    LOGGER.with(|cell| *cell.borrow_mut() = Some(Box::new(logger)))
}

pub fn flush() {
    LOGGER.with(|cell| {
        if let Some(ref logger) = cell.borrow().as_ref() {
            logger.flush();
        }
    })
}
