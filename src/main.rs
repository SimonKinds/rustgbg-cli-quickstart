use notify::Watcher;
use std::{env, path::PathBuf, sync::mpsc, time::Duration};

fn main() {
    let settings = parse_arguments();

    // Set up the filesystem watcher. Events will be send back over the `event_rx` channel receiver.
    let (event_tx, event_rx) = mpsc::channel();
    let mut watcher = notify::watcher(event_tx, settings.delay).unwrap();
    watcher
        .watch(settings.watch_path, notify::RecursiveMode::Recursive)
        .unwrap();

    let events_listened_for = &settings.events;

    for event in event_rx {
        use notify::DebouncedEvent::*;
        match event {
            NoticeWrite(_path) => (),
            NoticeRemove(_path) => (),
            Create(path) => if events_listened_for.contains(&Events::Create) {println!("create {}", path.display())},
            Write(path) => if events_listened_for.contains(&Events::Write) {println!("write {}", path.display())},
            Chmod(path) => if events_listened_for.contains(&Events::Chmod) {println!("chmod {}", path.display())},
            Remove(path) => if events_listened_for.contains(&Events::Remove) {println!("remove {}", path.display())},
            Rename(from, to) => if events_listened_for.contains(&Events::Rename) {println!("rename {} => {}", from.display(), to.display())},
            Rescan => (),
            Error(error, None) => eprintln!("error: {}", error),
            Error(error, Some(path)) => eprintln!("error at {}: {}", path.display(), error),
        }
    }
    // Since we want to listen to events until the user exits with ctrl-c, it's always an error to
    // reach the end of the program.
    eprintln!("Notify has crashed");
    ::std::process::exit(1);
}

#[derive(Eq,PartialEq)]
enum Events {
    Create,
    Write,
    Chmod,
    Remove,
    Rename
}

struct Settings {
    watch_path: PathBuf,
    delay: Duration,
    events: Vec<Events>
}

/// Uses the `clap` crate to generate help/usage printing as well as parse the given arguments.
fn parse_arguments() -> Settings {
    let possible_event_values = ["all", "create", "write", "chmod", "remove", "rename"];
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("PATH")
                .help("The path to watch. Uses current working directory if not specified")
                .index(1),
        )
        .arg(
            clap::Arg::with_name("delay")
                .help("Set the event delay in ms. Helps group chained events into single events")
                .short("d")
                .long("delay")
                .default_value("100"),
        )
        .arg(
            clap::Arg::with_name("operations")
            .help("Listen for the following operations")
            .short("o")
            .long("operation")
            .default_value("all")
            .multiple(true)
            .possible_values(&possible_event_values),
            )
        .get_matches();

    // Pull out the PATH argument. Fall back to the current working directory if it was not given.
    let watch_path = match matches.value_of_os("PATH") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().unwrap(),
    };

    // Get the delay value and try to parse it into an u64. If this fails clap will print an error
    // and make the program exit.
    let delay_ms = clap::value_t!(matches.value_of("delay"), u64).unwrap_or_else(|e| e.exit());
    let delay = Duration::from_millis(delay_ms);

    let mut events = Vec::new();
    for event_string in matches.values_of("operations").unwrap() {
        match event_string.as_ref() {
            "create" => events.push(Events::Create),
            "write" => events.push(Events::Write),
            "chmod" => events.push(Events::Chmod),
            "remove" => events.push(Events::Remove),
            "rename" => events.push(Events::Rename),
            "all" => {
                events.push(Events::Create);
                events.push(Events::Write);
                events.push(Events::Chmod);
                events.push(Events::Remove);
                events.push(Events::Rename);
            }
            _ => {}
        }
    }

    Settings { watch_path, delay, events }
}
