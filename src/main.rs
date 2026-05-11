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
    /// Encrypt a note by index
    Encrypt { index: usize },
    /// Decrypt a note by index
    Decrypt { index: usize },
}

fn search(query: &str) -> io::Result<()> {
    let directory = Path::new("Notes");
    let mut string_found = false;
    let querylowercase = query.to_lowercase();

    for (index, entry) in fs::read_dir(directory)?.enumerate() {
        let entry = entry?;
        let path = entry.path();
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let lowercase_line = line.to_lowercase();

            if lowercase_line.contains(&querylowercase) {
                string_found = true;
                println!("{} | {} | {}", index + 1, path.display(), line)
            }
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
    let entries: Vec<_> = std::fs::read_dir("Notes")?.filter_map(Result::ok).collect();
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();

        if path.is_file() {
            let contents = fs::read_to_string(&path)?;
            println!("{} | {} | {}", index + 1, path.display(), contents);
        }
    }
    Ok(())
}
fn delete(index: usize) -> std::io::Result<()> {
    let entries: Vec<_> = std::fs::read_dir("Notes")?.filter_map(Result::ok).collect();
    for (i, entry) in entries.iter().enumerate() {
        if i + 1 == index {
            fs::remove_file(entry.path())?;
            break;
        }
    }
    println!("{:?} ", entries.iter().enumerate());
    Ok(())
}
fn encrypt(index: usize) -> io::Result<()> {
    let key = b"k";
    let shift = 3;
    let entries: Vec<_> = std::fs::read_dir("Notes")?.filter_map(Result::ok).collect();
    if let Some(entry) = entries.get(index) {
        let path = entry.path();
        let data = fs::read(&path)?;

        let out: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, b)| (b ^ key[i % key.len()]).rotate_left(shift as u32))
            .collect();
        let enc_path = path.with_extension("enc");

        if path.extension().and_then(|ext| ext.to_str()) != Some("enc") {
            fs::remove_file(path)?;
            fs::write(&enc_path, &out)?;
        }
    }
    Ok(())
}
fn decrypt(index: usize) -> io::Result<()> {
    let key = b"k";
    let shift = 3;
    let entries: Vec<_> = std::fs::read_dir("Notes")?.filter_map(Result::ok).collect();
    if let Some(entry) = entries.get(index) {
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

        if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
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
        Commands::Decrypt { index } => decrypt(index)?,
        Commands::Encrypt { index } => encrypt(index)?,
    }

    Ok(())
}
