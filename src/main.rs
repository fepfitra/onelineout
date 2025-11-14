use std::env;
use std::io::{self, BufRead, Write};

fn parse_args() -> usize {
    let mut lines: usize = 1;
    let mut args = env::args().skip(1);

    while let Some(a) = args.next() {
        if a == "-l" || a == "--lines" {
            if let Some(n) = args.next() {
                if let Ok(v) = n.parse::<usize>() {
                    if v > 0 {
                        lines = v;
                    }
                }
            }
        } else if a.starts_with("-l=") {
            if let Some(nstr) = a.splitn(2, '=').nth(1) {
                if let Ok(v) = nstr.parse::<usize>() {
                    if v > 0 {
                        lines = v;
                    }
                }
            }
        } else if a.starts_with("--lines=") {
            if let Some(nstr) = a.splitn(2, '=').nth(1) {
                if let Ok(v) = nstr.parse::<usize>() {
                    if v > 0 {
                        lines = v;
                    }
                }
            }
        }
    }

    lines
}

fn main() -> io::Result<()> {
    let lines = parse_args();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let is_tty = atty::is(atty::Stream::Stdout);

    if !is_tty || lines == 0 {
        for line_res in stdin.lock().lines() {
            let line = line_res?;
            println!("{}", line);
        }
        return Ok(());
    }

    if lines == 1 {
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

        return Ok(());
    }

    let mut buffer: Vec<String> = Vec::with_capacity(lines);
    let mut prev_widths: Vec<usize> = Vec::new();
    let mut printed_count: usize = 0;

    for line_res in stdin.lock().lines() {
        let line = line_res?;

        buffer.push(line);
        if buffer.len() > lines {
            buffer.remove(0);
        }

        let cur_len = buffer.len();

        let cur_widths: Vec<usize> = buffer.iter().map(|s| s.chars().count()).collect();

        if printed_count > 0 {
            write!(stdout, "\x1B[{}A", printed_count)?; // ANSI: move up
        }

        for (i, s) in buffer.iter().enumerate() {
            let w = cur_widths[i];
            write!(stdout, "{}", s)?;

            if i < prev_widths.len() {
                if prev_widths[i] > w {
                    write!(stdout, "{}", " ".repeat(prev_widths[i] - w))?;
                }
            }

            write!(stdout, "\n")?;
        }

        if printed_count > cur_len {
            for i in cur_len..printed_count {
                let prev_w = prev_widths.get(i).copied().unwrap_or(0);
                write!(stdout, "{}\n", " ".repeat(prev_w))?;
            }
        }

        stdout.flush()?;

        prev_widths = cur_widths;
        printed_count = buffer.len();
    }

    Ok(())
}
