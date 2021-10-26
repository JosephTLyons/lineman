# TODO

- Fix all bad error handling - don't use string errors and don't use unwraps - some errors might be killing the program when the program could just continue on
- Better logging - Log what has been checked, what has actually been changed, and what couldn't be changed, for whatever reason
- Show numerical stats on how many files were looked at, how many were changed, duration of run, etc
- Tweak command line argument parsing (help, info, etc)
- Add flag to turn off newline normalization
- Make more efficient and clean (clean_file / clean_lines functions)
- Reimplement mechanism to only write to file if lines were changed and only report changed files
