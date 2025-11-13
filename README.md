# onelineout

Small Rust utility that reads lines from stdin and prints each line followed by a carriage return (\r), allowing the output to overwrite the current terminal line.

Behavior
- If a line contains only whitespace (or is empty), the program prints it normally with a newline.
- Otherwise the program prints the line followed by `\r` and clears any leftover characters from the previous longer line.

Examples (POSIX shell)

```sh
# Print two lines where the second is shorter; the program clears leftover characters
printf "aaaaaaaaaa\nbbbb\n" | onelineout

# Pipe output from another program
ls -lah | onelineout
```

Notes
- This is a tiny CLI intended to be used in pipelines. It is implemented in `src/main.rs`.
- The program flushes stdout after each non-blank line to make live updating behavior visible in the terminal.
