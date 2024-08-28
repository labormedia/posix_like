# Minimal Posix Like Shell written in Rust

## Design considerations.
Absolute paths.
Includes commands in $PATH.
Handles single and double quotes as inclosing characters.
Includes "cd" and "exit" (with exit code) commands.
Handles command's exit codes.
Makes use only of std library.
Command character lengths is 1000 max.
Arguments list size is 100 max.
Omits other features.

## Build
```
$ cargo build --release
```

## Run

```
$ ./target/release/posix_like
$
```