use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut prev_len: usize = 0;

    for line_res in stdin.lock().lines() {
        let line = line_res?;

        if line.trim().is_empty() {
            writeln!(stdout, "{}", line)?;
            prev_len = 0;
            continue;
        }

        let len = line.chars().count();

        write!(stdout, "\r{}", &line)?;
        if prev_len > len {
            write!(stdout, "{}", " ".repeat(prev_len - len))?;
            write!(stdout, "\r")?;
        } else {
            write!(stdout, "\r")?;
        }

        stdout.flush()?;
        prev_len = len;
    }

    Ok(())
}
