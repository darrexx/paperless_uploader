use std::{env, path::Path, sync::mpsc, thread};

use config::Config;
use error_logging::LogError;
use flexi_logger::{FileSpec, FlexiLoggerError, Logger};
use notify::{event::CreateKind, Event, EventKind, Watcher};
use serde::Deserialize;

use crate::document_processor::process_new_document;

pub mod document_processor;
pub mod error_logging;

#[derive(Deserialize, Debug)]
struct Configuration {
    token: String,
    url: String,
    folder: String,
    #[serde(default = "default_loglevel")]
    loglevel: String,
}

fn default_loglevel() -> String {
    "debug".to_string()
}

fn main() {
    let settings = load_config();
    let _logger = configure_logging(&settings.loglevel).expect("Error Creating Logger");

    log::info!("Initialization finised");
    log::debug!("Initialized with config: {:?}", &settings);

    let (sender, receiver) = mpsc::channel();

    let folder_path = &settings.folder;
    let mut watcher = notify::recommended_watcher(sender)
        .log_if_error("Error creating watcher")
        .expect("Error creating watcher");
    watcher
        .watch(Path::new(folder_path), notify::RecursiveMode::NonRecursive)
        .log_if_error("watch failed")
        .expect("watch failed");

    log::info!("Watch started for new files");

    loop {
        let res = receiver.recv();
        match res {
            Ok(Ok(Event {
                kind: EventKind::Create(CreateKind::File),
                paths,
                ..
            })) => {
                for path in paths {
                    log::debug!("New File Created Event for file: {:?}", path);
                    // println!("{:?}", std::fs::metadata(&path).unwrap().len()); //enable logging

                    let url = settings.url.clone();
                    let token = settings.token.clone();
                    let thread_id = thread::spawn(|| process_new_document(path, url, token))
                        .thread()
                        .id();
                    log::info!(
                        "created new Thread to process document with ThreadId {:?}",
                        thread_id
                    );
                }
            }
            Err(e) => log::error!("watch receive error: {}", e),
            _ => (),
        }
    }
}

fn load_config() -> Configuration {
    let mut config_path = env::current_exe().expect("Could not get exe directory");
    config_path.pop();
    config_path.push("config.json");

    Config::builder()
        .add_source(config::File::from(config_path))
        .build()
        .unwrap()
        .try_deserialize::<Configuration>()
        .unwrap()
}

fn configure_logging(log_level: &str) -> Result<flexi_logger::LoggerHandle, FlexiLoggerError> {
    let mut exe_folder = env::current_exe()?;
    exe_folder.pop();
    Logger::try_with_env_or_str(log_level)?
        .log_to_file(FileSpec::default().directory(exe_folder))
        .format(flexi_logger::detailed_format)
        .rotate(
            flexi_logger::Criterion::Age(flexi_logger::Age::Day),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogAndCompressedFiles(5, 30),
        )
        .start()
}
