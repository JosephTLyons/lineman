# lineman

Clean up the lines of files in your code repository

**NOTE**: While lineman does have tests in place to ensure it operates in a
specific way, I still caution any user to only use lineman on version-controlled
repositories, so that, in the event of some catastrophic failure, file changes
can be reverted.

---

Lineman currently performs two actions:

1. Strips trailing whitespace in every line in a file
2. Normalizes the newline count at the end of a file to one

The following command would run lineman on every rust and python file within the
`/path/to/some/repository` directory.

```shell
cargo run -- -p /path/to/some/repository -e rs py
```

Currently, there is a flag that will disable the end-of-file newline
normalization: `disable_eof_newline_normalization` or `d`

```shell
cargo run -- -p /path/to/some/repository -e rs py -d
```

The tests at the end of [main.rs](src/main.rs) show how lineman transforms a
file's content
