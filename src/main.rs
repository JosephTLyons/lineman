use std::ffi::OsStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct StwArgs {
    #[structopt(short, long)]
    extensions: Vec<String>,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    path: PathBuf,
}

fn main() -> Result<(), String> {
    // let untouched_files: Vec<String> = Vec::new();
    // let cleaned_files: Vec<String> = Vec::new();
    // let files_with_errors: Vec<String> = Vec::new();

    let args = StwArgs::from_args();

    for dir_entry_result in WalkDir::new(args.path) {
        match dir_entry_result {
            Ok(dir_entry) => {
                let path = dir_entry.path();

                if !path.is_file() {
                    continue;
                }

                if let Some(extension) = path.extension() {
                    if args
                        .extensions
                        .iter()
                        .map(|extension| OsStr::new(extension))
                        .any(|xtension| xtension == extension)
                    {
                        let path_display = path.display();

                        match clean_file(path) {
                            Ok(_) => println!("Cleaned: {}", path_display),
                            Err(_) => println!("Not cleaned: {}", path_display),
                        }
                    }
                }
            }
            Err(_) => return Err("Bad Path".to_string()),
        }
    }

    Ok(())
}

fn clean_file(path: &Path) -> Result<bool, String> {
    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut file_was_cleaned: bool = false;

    {
        let file = File::open(path).map_err(|_| format!("Cannot open file {}", path.display()))?;
        let buf_reader = BufReader::new(file);

        for line_result in buf_reader.lines() {
            match line_result {
                Ok(line) => {
                    let (cleaned_line, line_was_cleaned) = clean_line(&line);

                    if line_was_cleaned {
                        file_was_cleaned = true;
                    }

                    cleaned_lines.push(cleaned_line);
                }
                Err(_) => return Err("Can't read line".to_string()),
            }
        }
    }

    let mut file = File::create(path).map_err(|_| "Cannot open file".to_string())?;

    for line in cleaned_lines {
        file.write_all(line.as_bytes()).unwrap();
    }

    Ok(file_was_cleaned)
}

fn clean_line(line: &str) -> (String, bool) {
    let cleaned_line = format!("{}\n", line.trim_end());
    let line_was_cleaned = cleaned_line == line;
    (cleaned_line, line_was_cleaned)
}

#[test]
fn clean_bad_lines() {
    let input_output_lines_array = [
        // Remove spaces
        ("some code    \n", "some code\n"),
        // Keep indentation, remove spaces
        ("    some code    \n", "    some code\n"),
        // Remove tab
        ("some code\t\n", "some code\n"),
        // Keep indentation, remove tab
        ("    some code\t\n", "    some code\n"),
        // Add newline
        ("some code", "some code\n"),
        // Remove spaces, add newline
        ("some code    ", "some code\n"),
        // Remove spaces
        ("    \n", "\n"),
        // Remove spaces, add newline
        ("    ", "\n"),
    ];

    test_runner(&input_output_lines_array);
}

#[test]
fn skip_clean_good_lines() {
    let input_output_lines_array = [("some code\n", "some code\n"), ("\n", "\n")];

    test_runner(&input_output_lines_array);
}

#[allow(dead_code)]
fn test_runner(input_output_lines_array: &[(&str, &str)]) {
    for (input, output) in input_output_lines_array {
        assert_eq!(clean_line(*input).0, *output);
    }
}

// TODO:

// Fix all bad error handling - don't use string errors and don't use unwraps - some errors might be killing the program when the program could just continue on
// Better program name
// Better logging - Log what has been checked, what has actually been changed, and what couldn't be changed, for whatever reason
// Show numerical stats on how many files were looked at, how many were changed, duration of run, etc
// Tweak command line argument parsing (help, info, etc)
