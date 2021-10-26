use std::ffi::OsStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "lineman")]
struct LinemanArgs {
    /// The root path from which to begin processing
    #[structopt(short, long)]
    path: PathBuf,

    /// A list of file extensions that dictates which files are processed
    #[structopt(short, long)]
    extensions: Vec<String>,
}

fn main() -> Result<(), String> {
    // let untouched_files: Vec<String> = Vec::new();
    // let cleaned_files: Vec<String> = Vec::new();
    // let files_with_errors: Vec<String> = Vec::new();

    let args = LinemanArgs::from_args();
    let mut files_cleaned: u32 = 0;

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
                        files_cleaned += 1;

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

    println!("Files Cleaned: {}", files_cleaned);

    Ok(())
}

fn clean_file(path: &Path) -> Result<(), String> {
    let cleaned_lines: Vec<String>;

    {
        let file = File::open(path).map_err(|_| format!("Cannot open file {}", path.display()))?;
        let buf_reader = BufReader::new(file);

        cleaned_lines = buf_reader
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .map_err(|_| "Can't read line".to_string())?;
    }

    let mut file = File::create(path).map_err(|_| "Cannot open file".to_string())?;

    for line in clean_lines(&cleaned_lines) {
        file.write_all(line.as_bytes()).unwrap();
    }

    Ok(())
}

fn clean_lines(lines: &[String]) -> Vec<String> {
    let mut cleaned_lines: Vec<String> = lines
        .iter()
        .map(|line| format!("{}\n", line.trim_end()))
        .collect();

    // Normalize newlines at the end of the file to 1
    // This is very ugly code - find a more elegant way to do this
    let mut newline_count: usize = 0;

    for line in cleaned_lines.iter().rev() {
        if line == "\n" {
            newline_count += 1;
        } else {
            break;
        }
    }

    if newline_count > 0 {
        cleaned_lines = cleaned_lines[0..cleaned_lines.len() - newline_count].to_vec();
    }

    cleaned_lines
}

#[test]
// When a file's lines are read into this application, the newlines are consumed,
// so we must test a case that mimics that behavior
fn clean_lines_add_newlines() {
    let input_lines = [
        "def main():",
        "    print(\"Hello World\")",
        "",
        "if __name__ == \"__main__\":",
        "    main()",
    ];

    let output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    test_runner(&input_lines, &output_lines);
}

#[test]
fn clean_lines_with_trailing_spaces() {
    let input_lines = [
        "def main():   \n",
        "    print(\"Hello World\")    \n",
        "    \n",
        "if __name__ == \"__main__\":    \n",
        "    main()    \n",
    ];

    let output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    test_runner(&input_lines, &output_lines);
}

#[test]
fn clean_lines_with_trailing_tabs() {
    let input_lines = [
        "def main():\t\n",
        "    print(\"Hello World\")\t\n",
        "\t\n",
        "if __name__ == \"__main__\":\t\n",
        "    main()\t\n",
    ];

    let output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    test_runner(&input_lines, &output_lines);
}

#[test]
fn clean_lines_add_newline_to_end_of_file() {
    let input_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()",
    ];

    let output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    test_runner(&input_lines, &output_lines);
}

#[test]
fn clean_lines_remove_excessive_newlines_from_end_of_file() {
    let input_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
        "\n",
        "\n",
        "\n",
    ];

    let output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    test_runner(&input_lines, &output_lines);
}

#[allow(dead_code)]
fn test_runner(input_lines: &[&str], output_lines: &[&str]) {
    let input_lines: Vec<String> = input_lines.iter().map(|line| line.to_string()).collect();
    assert_eq!(clean_lines(&input_lines), output_lines);
}
