//! Provides the necessary utils to monitor logfile changes and return the as they're happening
//!

use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use futures::{
    channel::mpsc::{channel, Receiver},
    future, SinkExt, Stream, StreamExt, TryStreamExt,
};
use home::home_dir;
use log::{debug, error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{models::Log, parse_log_line};

/// Returns the log directory for EVE.
/// This should be the proper Windows path but checks that it's the same under Linux should be
/// done
pub fn get_log_folder() -> PathBuf {
    let mut path = home_dir().unwrap(); // home directory
    path.push("Documents");
    path.push("EVE");
    path.push("logs");
    path.push("Gamelogs");
    path
}

/// Returns all new logs sent to a file as a stream
pub async fn watch_log_file(logfile: PathBuf) -> impl Stream<Item = Log> {
    let (mut watcher, rx) = create_watcher().unwrap();

    watcher
        .watch(&logfile, RecursiveMode::NonRecursive)
        .unwrap();
    info!("Watcher started on file {:?}", &logfile);

    let mut log_contents = fs::read_to_string(&logfile).unwrap();
    let mut pos_in_file = log_contents.len() as u64;

    rx.into_stream()
        .inspect_err(|err| error!("Error in the watcher: {}", err))
        .filter(|element| {
            future::ready(match element {
                Ok(event) => filter_ok_events(event),
                _ => false,
            })
        })
        .map(move |_| -> Option<Log> {
            let mut f = File::open(&logfile).unwrap();
            f.seek(SeekFrom::Start(pos_in_file)).unwrap();

            pos_in_file = f.metadata().unwrap().len();

            log_contents.clear();
            f.read_to_string(&mut log_contents).unwrap();
            info!("new content: {}", log_contents);
            parse_log_line(&log_contents)
        })
        .filter_map(future::ready) // removes Nones from the stream
}

/// This function will filter events to keep only the events of type `Event::ModifyKind`
fn filter_ok_events(event: &Event) -> bool {
    if let notify::event::EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
        return true;
    }
    false
}

/// Initiates a watcher and the receiver with default configuration
fn create_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    debug!("Starting to create a watcher and the receiver");
    let (mut tx, rx) = channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}
