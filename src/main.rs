use std::ffi::OsStr;
use std::fs::{self, File};
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
    extensions: Option<Vec<String>>,

    /// Disables EOF newline normalization
    #[structopt(short, long)]
    disable_eof_newline_normalization: bool,
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

    let normalize_eof_newlines = !args.disable_eof_newline_normalization;

    for dir_entry_result in WalkDir::new(root_path) {
        match dir_entry_result {
            Ok(dir_entry) => {
                let path = dir_entry.path();

                if !path.is_file() {
                    continue;
                }

                if let Some(current_file_extension) = path.extension() {
                    let should_clean_file = args.extensions.as_ref().map_or(true, |extensions| {
                        extensions
                            .iter()
                            .any(|extension| OsStr::new(extension) == current_file_extension)
                    });

                    if should_clean_file {
                        match clean_file(path, normalize_eof_newlines) {
                            Ok(file_was_cleaned) => {
                                if file_was_cleaned {
                                    cleaned_file_paths.push(path.to_path_buf())
                                }
                            }
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

    print_report(&cleaned_file_paths, &skipped_file_paths, &walk_dir_errors);

    Ok(())
}

fn clean_file(path: &Path, normalize_eof_newlines: bool) -> Result<bool, LinemanFileError> {
    let file_string = fs::read_to_string(path).map_err(|_| LinemanFileError::FileNotOpened)?;
    let lines: Vec<&str> = file_string.split_inclusive('\n').collect();
    let (clean_lines, file_was_cleaned) = clean_lines(&lines, normalize_eof_newlines);

    if file_was_cleaned {
        let mut file = File::create(path).map_err(|_| LinemanFileError::FileNotCleaned)?;

        for clean_line in clean_lines {
            // TODO: This needs more thought, as a failure here means the file is probably only partially written to
            // Better hope your files are version controlled
            file.write_all(clean_line.as_bytes())
                .map_err(|_| LinemanFileError::FileNotCleaned)?;
        }
    }

    Ok(file_was_cleaned)
}

fn clean_lines(lines: &[&str], normalize_eof_newlines: bool) -> (Vec<String>, bool) {
    let mut cleaned_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            let line_has_newline = line.ends_with('\n');
            let trimmed_line = line.trim_end();
            let cleaned_line = if normalize_eof_newlines || line_has_newline {
                format!("{}\n", trimmed_line)
            } else {
                trimmed_line.to_string()
            };

            cleaned_line
        })
        .rev()
        .skip_while(|line| normalize_eof_newlines && line.trim_end().is_empty())
        .collect::<Vec<_>>();

    cleaned_lines.reverse();

    // This is probably slow, maybe a better method can be implemented later
    let lines_were_cleaned = lines != cleaned_lines;

    (cleaned_lines, lines_were_cleaned)
}

fn print_report(
    cleaned_file_paths: &[PathBuf],
    skipped_file_paths: &[PathBuf],
    walk_dir_errors: &[Error],
) {
    let indent = " ".repeat(4);

    if !cleaned_file_paths.is_empty() {
        println!("Cleaned Files:");

        for cleaned_file_path in cleaned_file_paths {
            println!("{}{}", indent, cleaned_file_path.display());
        }
    }

    if !skipped_file_paths.is_empty() {
        println!("Skipped Files:");

        for skipped_file_path in skipped_file_paths {
            println!("{}{}", indent, skipped_file_path.display());
        }
    }

    if !walk_dir_errors.is_empty() {
        println!("Walkdir Errors:");

        for walk_dir_error in walk_dir_errors {
            println!("{}{}", indent, walk_dir_error);
        }
    }
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, true);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, true);
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, true);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, true);
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, true);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, true);
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, false);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, false);
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, true);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, true);
}

#[test]
fn do_not_remove_excessive_newlines_from_end_of_file() {
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

    let expected_output_lines = [
        "def main():\n",
        "    print(\"Hello World\")\n",
        "\n",
        "if __name__ == \"__main__\":\n",
        "    main()\n",
        "\n",
        "\n",
        "\n",
    ];

    let (output_lines, lines_have_changes) = clean_lines(&input_lines, false);

    assert_eq!(expected_output_lines.to_vec(), output_lines);
    assert_eq!(lines_have_changes, false);
}
