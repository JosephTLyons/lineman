use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
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

#[derive(Debug)]
enum LinemanApplicationError {
    InvalidRootPath(String),
}

enum LinemanFileError {
    FileNotOpened,
    FileNotCleaned,
}

fn main() -> Result<(), LinemanApplicationError> {
    let mut cleaned_file_paths: Vec<PathBuf> = Vec::new();
    let mut skipped_file_paths: Vec<PathBuf> = Vec::new();
    let mut walk_dir_errors: Vec<Error> = Vec::new();

    let args = LinemanArgs::from_args();
    let root_path = args.path;

    if !root_path.is_dir() {
        return Err(LinemanApplicationError::InvalidRootPath(
            "The provided path is not a valid directory".to_string(),
        ));
    }

    let normalize_eof_newlines = false;

    for dir_entry_result in WalkDir::new(root_path) {
        match dir_entry_result {
            Ok(dir_entry) => {
                let path = dir_entry.path();

                if !path.is_file() {
                    continue;
                }

                if let Some(current_file_extension) = path.extension() {
                    if args
                        .extensions
                        .iter()
                        .any(|extension| current_file_extension == OsStr::new(extension))
                    {
                        // TODO: Find a way to not have to convert to PathBuf
                        match clean_file(path, normalize_eof_newlines) {
                            Ok(_) => cleaned_file_paths.push(path.to_path_buf()),
                            Err(
                                LinemanFileError::FileNotOpened | LinemanFileError::FileNotCleaned,
                            ) => skipped_file_paths.push(path.to_path_buf()),
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

    Ok(())
}

fn clean_file(path: &Path, normalize_eof_newlines: bool) -> Result<(), LinemanFileError> {
    let file_string = fs::read_to_string(path).map_err(|_| LinemanFileError::FileNotOpened)?;
    let lines: Vec<&str> = file_string.split_inclusive('\n').collect();
    let mut file = File::create(path).map_err(|_| LinemanFileError::FileNotCleaned)?;

    for clean_line in clean_lines(&lines, normalize_eof_newlines) {
        // TODO: This needs more thought, as a failure here means the file is probably only partially written to
        file.write_all(clean_line.as_bytes())
            .map_err(|_| LinemanFileError::FileNotCleaned)?;
    }

    Ok(())
}

fn clean_lines(lines: &[&str], normalize_eof_newlines: bool) -> Vec<String> {
    let mut cleaned_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            let line_has_newline = line.ends_with('\n');
            let trimmed_line = line.trim_end();

            if normalize_eof_newlines || line_has_newline {
                return format!("{}\n", trimmed_line);
            }

            trimmed_line.to_string()
        })
        .rev()
        .skip_while(|line| normalize_eof_newlines && line.trim_end().is_empty())
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

    test_runner(&input_lines, &output_lines, true);
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

    test_runner(&input_lines, &output_lines, true);
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

    test_runner(&input_lines, &output_lines, true);
}

#[test]
fn do_not_add_newline_to_end_of_file() {
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
        "    main()",
    ];

    test_runner(&input_lines, &output_lines, false);
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

    test_runner(&input_lines, &output_lines, true);
}

#[test]
fn keep_excessive_newlines_from_end_of_file() {
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
        "\n",
        "\n",
        "\n",
    ];

    test_runner(&input_lines, &output_lines, false);
}

#[allow(dead_code)]
fn test_runner(input_lines: &[&str], output_lines: &[&str], normalize_eof_newlines: bool) {
    assert_eq!(clean_lines(input_lines, normalize_eof_newlines), output_lines);
}
