use chrono::prelude::*;
use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Parser)]
#[command(about = " /////////////////
 A note taking app
 \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for a note
    Search { query: String },
    /// Make a new note
    Add { query: String },
    /// Init a Notes folder
    Init,
    /// List all notes and their indexes
    List,
    /// Delete a note by index
    Delete { index: usize },
    /// Obfuscate a note by index
    Obfuscate { index: usize },
    /// Deobfuscate a note by index
    Deobfuscate { index: usize },
    /// Deletes all notes
    DeleteAll,
}

fn is_obfuscated(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("enc")
}

fn read_entries() -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir("Notes")? {
        match entry {
            Ok(x) => entries.push(x),
            Err(x) => {
                eprintln!("Entry {} not readable", x)
            }
        }
    }
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

fn search(query: &str) -> io::Result<()> {
    let mut string_found = false;
    let query_lowercase = query.to_lowercase();
    let entries = read_entries()?;

    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        if is_obfuscated(&path) {
            continue;
        }
        let mut matched = false;
        for line in reader.lines() {
            let line = line?;
            let lowercase_line = line.to_lowercase();

            if lowercase_line.contains(&query_lowercase) {
                matched = true;
                string_found = true;
            }
        }
        if matched {
            let contents = fs::read_to_string(&path)?;

            println!("{} | {} | {}", index + 1, path.display(), contents);
        }
    }
    if !string_found {
        println!("String '{}' not present", query);
    }
    Ok(())
}

fn init() -> std::io::Result<()> {
    if Path::new("Notes").exists() {
        println!("Folder already exists")
    } else {
        fs::create_dir("Notes")?;
    }
    Ok(())
}

fn add(query: &str) -> io::Result<()> {
    let filename = Local::now().format("%Y-%m-%d_%H-%M-%S.txt").to_string();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("Notes/{}", filename))?;
    write!(file, "{}", query)?;
    Ok(())
}
fn list() -> io::Result<()> {
    let entries = read_entries()?;
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();

        if is_obfuscated(&path) {
            println!("{} | {} | OBFUSCATED", index + 1, path.display());
        } else {
            let contents = fs::read_to_string(&path)?;
            println!("{} | {} | {}", index + 1, path.display(), contents);
        }
    }
    Ok(())
}
fn delete_all() -> std::io::Result<()> {
    let entries = read_entries()?;
    for entry in entries {
        fs::remove_file(entry.path())?;
    }
    Ok(())
}
fn delete(index: usize) -> std::io::Result<()> {
    let entries = read_entries()?;
    for (i, entry) in entries.iter().enumerate() {
        if i + 1 == index {
            fs::remove_file(entry.path())?;
            break;
        }
    }
    Ok(())
}
fn obfuscate(index: usize) -> io::Result<()> {
    let key = b"k";
    let shift = 3;
    let entries = read_entries()?;
    if let Some(entry) = entries.get(index - 1) {
        let path = entry.path();
        let data = fs::read(&path)?;

        let out: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, b)| (b ^ key[i % key.len()]).rotate_left(shift as u32))
            .collect();
        let enc_path = path.with_extension("enc");

        if !is_obfuscated(&path) {
            fs::remove_file(path)?;
            fs::write(&enc_path, &out)?;
        }
    }
    Ok(())
}
fn deobfuscate(index: usize) -> io::Result<()> {
    let key = b"k";
    let shift = 3;
    let entries = read_entries()?;
    if let Some(entry) = entries.get(index - 1) {
        let path = entry.path();
        let data = fs::read(&path)?;

        let out: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let rotated = b.rotate_right(shift);
                rotated ^ key[i % key.len()]
            })
            .collect();
        let dec_path = path.with_extension("txt");

        if is_obfuscated(&path) {
            fs::remove_file(path)?;
            fs::write(dec_path, &out)?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Search { query } => search(&query)?,
        Commands::Add { query } => add(&query)?,
        Commands::Init => init()?,
        Commands::List => list()?,
        Commands::Delete { index } => delete(index)?,
        Commands::Obfuscate { index } => obfuscate(index)?,
        Commands::Deobfuscate { index } => deobfuscate(index)?,
        Commands::DeleteAll => delete_all()?,
    }

    Ok(())
}
