use std::ffi::OsStr;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::{Error, WalkDir};

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

enum LinemanError {
    FileNotOpened,
    FileNotCleaned,
}

fn main() {
    let mut cleaned_file_paths: Vec<PathBuf> = Vec::new();
    let mut skipped_file_paths: Vec<PathBuf> = Vec::new();
    let mut walk_dir_errors: Vec<Error> = Vec::new();

    let args = LinemanArgs::from_args();

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
                        .any(|xtension| OsStr::new(xtension) == extension)
                    {
                        // TODO: Find a way to not have to convert to PathBuf
                        match clean_file(path) {
                            Ok(_) => cleaned_file_paths.push(path.to_path_buf()),
                            Err(LinemanError::FileNotOpened | LinemanError::FileNotCleaned) => {
                                skipped_file_paths.push(path.to_path_buf())
                            }
                        }
                    }
                }
            }
            // TODO: I don't really know what the hell this error is, so I'm just grabbing it and printing it at the end in the report.
            // When I have a better idea of what it is, I can do something different, I guess
            Err(walk_dir_error) => walk_dir_errors.push(walk_dir_error),
        }
    }

    let category_and_file_paths = [
        (("Cleaned Files:"), cleaned_file_paths),
        (("Skipped Files:"), skipped_file_paths),
    ];

    for (category, file_paths) in category_and_file_paths {
        println!("{}", category);

        for file_path in file_paths {
            println!("{}{}", " ".repeat(4), file_path.display());
        }
    }

    println!("Walkdir Errors:");

    for walk_dir_error in walk_dir_errors {
        println!("{}{}", " ".repeat(4), walk_dir_error);
    }
}

fn clean_file(path: &Path) -> Result<(), LinemanError> {
    let lines: Vec<String>;

    {
        let file = File::open(path).map_err(|_| LinemanError::FileNotOpened)?;
        let buf_reader = BufReader::new(file);

        // Need to add newline to each line because the original newline is consumed when collecting the lines from the file
        // Is there a better way to do this where we don't have to manually add them back in?
        lines = buf_reader
            .lines()
            .map(|line_result| line_result.map(|line| line + "\n"))
            .collect::<Result<Vec<String>, _>>()
            .map_err(|_| LinemanError::FileNotCleaned)?;
    }

    let mut file = File::create(path).map_err(|_| LinemanError::FileNotCleaned)?;

    for clean_line in clean_lines(&lines) {
        // TODO: This needs more thought, as a failure here means the file is probably only partially written to
        file.write_all(clean_line.as_bytes())
            .map_err(|_| LinemanError::FileNotCleaned)?;
    }

    Ok(())
}

fn clean_lines(lines: &[String]) -> Vec<String> {
    let mut cleaned_lines: Vec<String> = lines
        .iter()
        .map(|line| format!("{}\n", line.trim_end()))
        .rev()
        .skip_while(|line| line.trim_end().is_empty())
        .collect::<Vec<_>>();

    cleaned_lines.reverse();
    cleaned_lines
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
fn add_newline_to_end_of_file() {
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
fn remove_excessive_newlines_from_end_of_file() {
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
