use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;
use exif::Tag;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Source folder, where not so well organized media live
    source: PathBuf,

    /// Target folder where image should be copied to, default to <SOURCE> (move files instead)
    target: Option<PathBuf>,

    /// Don't actually rename
    #[arg(short, long)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();
    println!("Organizing {}...", args.source.to_string_lossy());

    if !args.source.exists() {
        eprintln!(
            "Folder {} does not exists!",
            args.source.to_str().unwrap_or("Invalid path")
        );
        process::exit(1);
    }

    let target = args.target.unwrap_or(args.source.clone());
    let action: Action = match args.source == target {
        true => Action::Move,
        false => Action::Copy,
    };
    let destination_path = add_trailing_slash(target);

    let options = Options {
        action,
        do_action: !args.dry_run,
        destination_path,
    };

    rename_files(args.source, &options);

    println!("Done!");
}

fn rename_files(folder_path: PathBuf, options: &Options) {
    for file in fs::read_dir(&folder_path).unwrap() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_dir() {
            rename_files(file.path(), &options);
            continue;
        }

        let file_data = parse_file(file.path());

        let filename = file_data
            .created_at
            .format("%Y/%Y%m%d/%Y-%m-%d_%H%M%S.")
            .to_string();
        let extension = file
            .path()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase();
        let path = match file_data.brightness.as_str() {
            "" => filename + &extension,
            _ => filename + &file_data.brightness + "." + &extension,
        };

        let full_destination_path = PathBuf::from(format!("{}{}", options.destination_path, path));

        // File doesn't need to be moved
        if full_destination_path == file.path() {
            continue;
        }
        // Destination file exists, skipping
        if matches!(options.action, Action::Copy) && full_destination_path.exists() {
            continue;
        }
        let action_name = match options.action {
            Action::Copy => "Copying",
            Action::Move => "Moving",
        };

        println!(
            "{} file {:?} to {:?}",
            action_name,
            file.path(),
            full_destination_path
        );
        if options.do_action {
            // Create directory if needed
            let mut directory = full_destination_path.clone();
            directory.pop();
            fs::create_dir_all(directory).unwrap();

            // move file
            match options.action {
                Action::Move => fs::rename(file.path(), full_destination_path).unwrap(),
                Action::Copy => copy_file(&file.path(), &full_destination_path),
            }
        }
    }

    // remove empty directory
    if folder_path.read_dir().unwrap().next().is_none() {
        println!("Deleting empty directory {:?}", folder_path);
        if options.do_action {
            fs::remove_dir(folder_path).unwrap();
        }
    }
}

fn add_trailing_slash(path: PathBuf) -> String {
    let path = path.to_str().unwrap().to_string();
    if path.ends_with('/') {
        return path;
    }

    return path + "/";
}

fn copy_file(src: &PathBuf, destination: &PathBuf) {
    fs::copy(src, destination).unwrap();
    let src_metadata = src.metadata().unwrap();
    let src_mtime = filetime::FileTime::from_last_modification_time(&src_metadata);
    filetime::set_file_mtime(&destination, src_mtime).unwrap();
}

enum Action {
    Copy,
    Move,
}
struct Options {
    do_action: bool,
    action: Action,
    destination_path: String,
}

struct FileMeta {
    pub created_at: NaiveDateTime,
    pub brightness: String,
}

fn parse_file(path: PathBuf) -> FileMeta {
    let mut file_meta = FileMeta {
        created_at: NaiveDateTime::default(),
        brightness: "".to_string(),
    };
    let file = std::fs::File::open(path.clone()).unwrap();
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    // Try exif first
    if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
        for field in exif.fields() {
            match field.tag {
                Tag::DateTimeOriginal => {
                    let string_date = field.value.display_as(Tag::DateTimeOriginal).to_string();
                    file_meta.created_at = match NaiveDateTime::parse_from_str(
                        string_date.as_str(),
                        "%Y-%m-%d %H:%M:%S",
                    ) {
                        Ok(date) => date,
                        Err(error) => {
                            panic!("Error parsing Exif date {:?}: {:?}", string_date, error)
                        }
                    };
                }
                Tag::BrightnessValue => {
                    file_meta.brightness = field.display_value().to_string();
                }
                _ => (),
            }
        }
    }
    if file_meta.created_at == NaiveDateTime::default() {
        // Fall back to metadata created at date
        let metadata = fs::metadata(path).unwrap();
        let date_created = metadata.modified().unwrap();
        let dt_now_utc: DateTime<Utc> = date_created.clone().into();

        file_meta.created_at = dt_now_utc.naive_local();
    }

    file_meta
}
