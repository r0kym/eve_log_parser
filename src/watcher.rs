//! Provides the necessary utils to monitor logfile changes and return the as they're happening
//!

use std::{fs::{self, File}, io::{Read, Seek, SeekFrom}, path::PathBuf};

use futures::{future, channel::mpsc::{channel, Receiver}, SinkExt, Stream, StreamExt, TryStreamExt};
use home::home_dir;
use notify::{event::DataChange, Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use log::{error, info};

use crate::{models::Log, parse_log_line};


/// Returns the log directory for EVE.
/// This should be the proper Windows path but checks that it's the same under Linux should be
/// done
pub fn get_log_folder() -> PathBuf {
    let mut path = home_dir().unwrap(); // home directory
    path.push("EVE");
    path.push("logs");
    path.push("gamelogs");
    path
    
}

/// Returns all new logs sent to a file as a stream
pub async fn watch_log_file(logfile: PathBuf) -> impl Stream<Item = Log> {

    let (mut watcher, rx) = create_watcher().unwrap();

    watcher.watch(&logfile, RecursiveMode::NonRecursive).unwrap();
    info!("Watcher started on file {:?}", &logfile);

    let mut log_contents = fs::read_to_string(&logfile).unwrap();
    let mut pos_in_file = log_contents.len() as u64;

    rx
        .into_stream()
        .inspect_err(|err| error!("Error in the watcher: {}", err))
        .filter(|element| future::ready(match element {
            Ok(event) => filter_ok_events(event),
            _ => false,
        }))
        .map(move |_| -> Option<Log> {
            let mut f = File::open(&logfile).unwrap();
            f.seek(SeekFrom::Start(pos_in_file)).unwrap();

            pos_in_file = f.metadata().unwrap().len();

            log_contents.clear();
            f.read_to_string(&mut log_contents).unwrap();
            info!("new content: {}", log_contents);
            parse_log_line(&log_contents)
        })
        .filter_map(|log| future::ready(log)) // removes Nones
            
}

/// This function will filter events to keep only the events of type `Event::ModifyKind`
fn filter_ok_events(event: &Event) -> bool {
    if let notify::event::EventKind::Modify(modifiy_event) = event.kind {
        if let notify::event::ModifyKind::Data(_) =  modifiy_event{
            return true; 
        }
    }
    false
}

/// Initiates a watcher and the receiver with default configuration
fn create_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        }, 
        Config::default()
    )?;

    Ok((watcher, rx))

}

