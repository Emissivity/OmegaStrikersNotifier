use clap::{ArgAction, Parser};
use notify::{Config, RecommendedWatcher, Watcher};
use notify_rust::Notification;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error, Seek},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

//Steam app ID for Omega Strikers
const APP_ID: i32 = 1869590;

/// Trails Omega Striker's log file to detect when a match is started and sends a notification
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Trails Omega Striker's log file to detect when a match is started and sends a notification."
)]
struct Args {
    /// Omega Strikers log file directory. If unset, the program will attempt to automatically find it.
    #[arg(short, long)]
    log_path: Option<PathBuf>,

    /// Polling frequency to check for updates to the file in seconds.
    #[arg(short, long, default_value_t = 5)]
    update_frequency: u64,

    /// Enables verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    debug: bool,
}

/// Entry point
fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let log_path = match &args.log_path {
        Some(path) => path,
        None => &find_log_file()?,
    };
    if args.debug {
        println!("Attempting with log file path: {:?}", log_path);
    }
    start_listening(&args, log_path)
}

/// Start monitoring for changes to the log file
fn start_listening(args: &Args, log_path: &PathBuf) -> Result<(), std::io::Error> {
    let (sender, reciever) = mpsc::channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(args.update_frequency));
    let mut tail = RecommendedWatcher::new(sender, config).expect("This should work");
    let _ = tail.watch(&log_path, notify::RecursiveMode::NonRecursive);
    let file = File::options().read(true).write(false).open(log_path)?;
    let mut reader = BufReader::new(file);
    let mut pos: usize = reader
        .seek(std::io::SeekFrom::End(0))?
        .try_into()
        .expect("This should not fail");
    println!("Listening to events");

    for res in reciever {
        match res {
            Ok(event) => {
                if args.debug {
                    println!("Recieved file event: {:?}", event.kind);
                }
                //skip events that aren't modifying the file
                //if event.kind.is_modify() || args.debug {
                    let mut line = String::new();
                    //read all remaining lines in case an event was missed
                    let mut line_count = 0;
                    loop {
                        let count_read = reader.read_line(&mut line).unwrap();
                        
                        pos += count_read;
                        if line.contains("\"state\":\"STARTING_GAME\"") {
                            println!("Line matched: {:?}", line.get(150..229));
                        }
                        match line.get(150..229) {
                            Some(
                                "Matchmaking Status: {\"state\":\"StartingGame\",\"idle\":{\"timestamp\":\"\",\"state\":\"\"},",
                            ) => found_match(),
                            Some(_) | None => (),
                        }
                        line.clear();
                        if count_read == 0 {
                            break;
                        }
                        line_count += 1;
                    }
                    if args.debug {
                        println!("Read {} lines", line_count);
                    }
                //}
            }
            Err(e) => {
                eprint!("{}", e);
            }
        }
    }
    Ok(())
}

/// Send a notification that you found a match
fn found_match() {
    let _ = Notification::new()
        .summary("Match Found!")
        .body("KO Them!")
        .icon("steam_icon_1869590")
        .appname("Omega Strikers Notifier")
        .show();
}

/// Get the path to the log file<br>
/// <h1>Unix-based systems</h1>This will locate the game's installation directory through Steam's <code>libraryfolders.vdf</code> to locate the correct <code>compatdata</code> folder.
/// <h1>Windows systems</h1>This will return the log file from the local appdata folder
pub fn find_log_file() -> Result<PathBuf, Error> {
    if cfg!(unix) {
        let libraryfolders = std::env::home_dir()
            .expect("Home directory should be locatable")
            .join(".local/share/Steam/config/libraryfolders.vdf");
        let file = match File::options().read(true).write(false).open(libraryfolders) {
            Ok(file) => file,
            Err(e) => return Err(e),
        };
        let reader = BufReader::new(file);
        let mut storage_location = PathBuf::new();
        for line in reader.lines().map(|l| l.unwrap()) {
            let trimmed = line.trim().replace("\"", "");
            if trimmed.is_empty() {
                continue;
            }
            let delimiter_pos = &trimmed.find("\t\t").unwrap_or_default();
            let key = trimmed
                .get(0..*delimiter_pos)
                .expect("This should always have a value");
            let value = trimmed.get(*delimiter_pos + 2..);

            if key == "path" {
                //set the current library folder to the found value
                storage_location = value
                    .expect("Path should always have a defined value")
                    .into();
            } else if key == APP_ID.to_string() {
                //exit the loop if omega strikers is found in the current library folder
                break;
            }
        }
        println!("Found installation directory at {:?}", storage_location);
        storage_location.push("steamapps/compatdata/1869590/pfx/drive_c/users/steamuser/AppData/Local/OmegaStrikers/Saved/Logs/OmegaStrikers.log");
        println!("Found log file at {:?}", storage_location);
        return Ok(storage_location);
    } else if cfg!(windows) {
        return Ok(std::env::home_dir()
            .expect("This shouldn't fail")
            .join("\\AppData\\Local\\OmegaStrikers\\Saved\\Logs\\OmegaStrikers.log"));
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported operating system!",
        ));
    }
}
