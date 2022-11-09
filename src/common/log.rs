use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::console::ConsoleAppender,
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
use std::env;

pub fn setup_log(
    log_level: Option<LevelFilter>,
    log_file_path: Option<String>,
) -> Result<Handle, SetLoggerError> {
    let log_level = match log_level {
        Some(l) => l,
        None => LevelFilter::Info,
    };

    let pattern_encoder = Box::new(PatternEncoder::new(
        "{d(%Y-%m-%d %H:%M:%S%.6f)} [{l}] {f}:{L} {T} - {m}\n",
    ));

    let appender_name = "appender";

    let mut config_builder = match log_file_path {
        Some(path) => {
            let logfile = FileAppender::builder()
                .encoder(pattern_encoder)
                .build(path)
                .unwrap();
            Config::builder().appender(Appender::builder().build(appender_name, Box::new(logfile)))
        }
        None => {
            let console = ConsoleAppender::builder().encoder(pattern_encoder).build();
            Config::builder().appender(Appender::builder().build(appender_name, Box::new(console)))
        }
    };

    let mut root_builder = Root::builder().appender(appender_name);

    let config = config_builder.build(root_builder.build(log_level)).unwrap();
    log4rs::init_config(config)
}
