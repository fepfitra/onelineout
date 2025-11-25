use std::env;
use std::io::{self, BufRead, Write};

fn parse_args() -> (usize, Option<String>) {
    let mut lines: usize = 1;
    let mut skip_at: Option<String> = None;
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
        } else if a == "--skip-at" {
            if let Some(val) = args.next() {
                skip_at = Some(val);
            }
        } else if a.starts_with("--skip-at=") {
            if let Some(val) = a.splitn(2, '=').nth(1) {
                skip_at = Some(val.to_string());
            }
        }
    }

    (lines, skip_at)
}

fn main() -> io::Result<()> {
    let (lines, skip_at) = parse_args();

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

            if let Some(marker) = &skip_at {
                if line.contains(marker) {
                    writeln!(stdout)?;
                    stdout.flush()?;
                    prev_len = 0;
                    continue;
                }
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
    //for testing release

    let mut buffer: Vec<String> = Vec::with_capacity(lines);
    let mut prev_widths: Vec<usize> = Vec::new();
    let mut printed_count: usize = 0;

    for line_res in stdin.lock().lines() {
        let line = line_res?;

        if let Some(marker) = &skip_at {
            if line.contains(marker) {
                if printed_count > 0 {
                    write!(stdout, "\x1B[{}A", printed_count)?;

                    for i in 0..printed_count {
                        let prev_w = prev_widths.get(i).copied().unwrap_or(0);
                        write!(stdout, "{}\n", " ".repeat(prev_w))?;
                    }
                    stdout.flush()?;
                }

                buffer.clear();
                prev_widths.clear();
                printed_count = 0;
                continue;
            }
        }

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

#[cfg(test)]
fn simulate(input: &[&str], lines: usize, skip_at: Option<&str>) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    if lines == 0 {
        return output;
    }

    if lines == 1 {
        let mut current: Option<String> = None;
        for &raw in input {
            if let Some(marker) = skip_at {
                if raw.contains(marker) {
                    if let Some(ref c) = current {
                        output.push(c.clone());
                    }
                    output.push(String::from(""));
                    current = None;
                    continue;
                }
            }
            current = Some(raw.to_string());
        }
        if let Some(c) = current {
            output.push(c);
        }
        return output;
    }

    let mut buffer: Vec<String> = Vec::new();
    for &raw in input {
        if let Some(marker) = skip_at {
            if raw.contains(marker) {
                if !buffer.is_empty() {
                    output.extend(buffer.drain(..));
                }
                output.push(String::from(""));
                continue;
            }
        }
        buffer.push(raw.to_string());
        if buffer.len() > lines {
            buffer.remove(0);
        }
    }
    if !buffer.is_empty() {
        output.extend(buffer.into_iter());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::simulate;

    #[test]
    fn single_line_overwrite_basic() {
        let input = ["aaaaaa", "bbb", "cccccccc", "ddd"];
        let out = simulate(&input, 1, None);
        assert_eq!(out, vec!["ddd"]);
    }

    #[test]
    fn single_line_with_skip_marker() {
        let input = ["first", "second", "MARK", "third", "fourth"];
        let out = simulate(&input, 1, Some("MARK"));
        assert_eq!(out, vec!["second", "", "fourth"]);
    }

    #[test]
    fn rolling_window_basic() {
        let input = ["L1", "L2", "L3", "L4", "L5"];
        let out = simulate(&input, 3, None);
        assert_eq!(out, vec!["L3", "L4", "L5"]);
    }

    #[test]
    fn rolling_window_with_skip() {
        let input = ["A1", "A2", "A3", "SKIP", "B1", "B2"];
        let out = simulate(&input, 3, Some("SKIP"));
        assert_eq!(out, vec!["A1", "A2", "A3", "", "B1", "B2"]);
    }

    #[test]
    fn rolling_window_multiple_skips() {
        let input = ["X1", "X2", "SKIP", "Y1", "Y2", "Y3", "Y4", "SKIP", "Z1"];
        let out = simulate(&input, 2, Some("SKIP"));
        assert_eq!(out, vec!["X1", "X2", "", "Y3", "Y4", "", "Z1"]);
    }
}
