use std::{fs::{self, File}, io::Write, path::PathBuf, sync::{Mutex, OnceLock}};

use log::{Level, LevelFilter, Log};

use time::OffsetDateTime;

use crate::config::get_global;

struct FeltyLogger {
    file: Option<Mutex<File>>,
}
impl FeltyLogger {
    fn new(log_directory: &str) -> FeltyLogger {
        let datetime = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let log_path = format!("{}/{}{:0>2}{:0>2}_{:0>2}{:0>2}{:0>2}.log",
            log_directory,
            datetime.year(), datetime.month() as u8, datetime.day(),
            datetime.hour(), datetime.minute(), datetime.second());

        let _ = fs::create_dir_all(log_directory);

        let file = match File::create_new(log_path) {
            Ok(file) => Some(Mutex::new(file)),
            Err(err) => {
                eprintln!("Could not create log file: {}", err.to_string());
                None
            },
        };
        FeltyLogger { file }
    }
}
impl Log for FeltyLogger {
    #[inline]
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if record.target().starts_with(env!("CARGO_CRATE_NAME")) {
            let datetime = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
            let module_path = record.module_path().unwrap_or("?");
            let file = record.file().unwrap_or("?");
            let line = record.line().map(|l| l.to_string()).unwrap_or_else(|| "?".to_string());

            let debug_info = if record.level() == Level::Trace {
                format!(" ({}:{})", file, line)
            } else if record.level() == Level::Debug {
                format!(" ({})", module_path)
            } else {
                "".to_string()
            };

            // NOTE: コンソールに出力
            println!("{:0>2}:{:0>2}:{:0>2}.{:0>4} {}{:>7}\x1b[0m{} {}",
                datetime.hour(), datetime.minute(), datetime.second(), datetime.millisecond(),
                match record.level() {
                    Level::Error => "\x1b[0;31m",
                    Level::Warn => "\x1b[0;33m",
                    Level::Info => "\x1b[0;34m",
                    Level::Debug => "\x1b[0;36m",
                    Level::Trace => "\x1b[0;37m",
                },
                format!("[{}]", record.level().as_str()),
                debug_info,
                record.args().to_string(),
            );

            // NOTE: ファイルに出力
            if let Some(ref file_mutex) = self.file {
                if let Ok(mut file) = file_mutex.lock() {
                    let _ = file.write_all(format!("{:0>2}:{:0>2}:{:0>2}.{:0>4} {:>7}\x1b[0m{} {}\n",
                        datetime.hour(), datetime.minute(), datetime.second(), datetime.millisecond(),
                        format!("[{}]", record.level().as_str()),
                        debug_info,
                        record.args().to_string(),
                    ).as_bytes());
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = file.flush();
            }
        }
    }
}

static LOGGER: OnceLock<FeltyLogger> = OnceLock::new();

pub fn setup_log() {
    let config = get_global();

    let _ = fs::create_dir_all(&config.log_directory);

    let logger = LOGGER.get_or_init(|| FeltyLogger::new(&config.log_directory));
    log::set_logger(logger).unwrap();
    log::set_max_level(LevelFilter::max());

    // NOTE: 古いログを削除する
    if let Ok(dir) = fs::read_dir(&config.log_directory) {
        let entry_results = dir.filter_map(|result| {
            if let Ok(entry) = result {
                if entry.metadata().is_ok_and(|d| d.len() == 0) {
                    let _ = fs::remove_file(entry.path());
                    return None;
                }

                if entry.file_type().is_ok_and(|t| t.is_file()) {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        return Some(file_name);
                    }
                }
            }
            None
        }).collect::<Vec<_>>();

        if entry_results.len() > 10 {
            for entry_result in &entry_results[..entry_results.len() - 10] {
                let _ = fs::remove_file(PathBuf::new().join(&config.log_directory).join(entry_result));
            }
        }
    }

    // NOTE: panic フックを設定
    std::panic::set_hook(Box::new(move |info| {
        let message = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "?",
            },
        };

        match info.location() {
            Some(location) => {
                log::error!(
                    "({}:{}) {}",
                    location.file(),
                    location.line(),
                    message,
                );
            }
            None => log::error!("{}", message),
        }
    }));

    log::info!("{} (v{})", config.name, config.version);
}
