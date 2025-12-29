use memmap2::Mmap;
use regex::bytes::{Regex, RegexBuilder};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process;

struct Config {
    pattern: String,
    files: Vec<String>,
    ignore_case: bool,
    invert_match: bool,
    count: bool,
    line_number: bool,
    files_with_matches: bool,
    files_without_match: bool,
    no_filename: bool,
    with_filename: bool,
    only_matching: bool,
    quiet: bool,
    recursive: bool,
    max_count: Option<usize>,
    fixed_strings: bool,
    word_regexp: bool,
    line_regexp: bool,
    color: bool,
    byte_offset: bool,
}

impl Config {
    fn from_args() -> Result<Config, String> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Err("Usage: rgrep [OPTIONS] PATTERN [FILE...]".to_string());
        }

        let mut ignore_case = false;
        let mut invert_match = false;
        let mut count = false;
        let mut line_number = false;
        let mut files_with_matches = false;
        let mut files_without_match = false;
        let mut no_filename = false;
        let mut with_filename = false;
        let mut only_matching = false;
        let mut quiet = false;
        let mut recursive = false;
        let mut max_count: Option<usize> = None;
        let mut fixed_strings = false;
        let mut word_regexp = false;
        let mut line_regexp = false;
        let mut color = atty::is(atty::Stream::Stdout);
        let mut byte_offset = false;

        let mut pattern = String::new();
        let mut files = Vec::new();
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "-i" | "--ignore-case" => ignore_case = true,
                "-v" | "--invert-match" => invert_match = true,
                "-c" | "--count" => count = true,
                "-n" | "--line-number" => line_number = true,
                "-l" | "--files-with-matches" => files_with_matches = true,
                "-L" | "--files-without-match" => files_without_match = true,
                "-h" | "--no-filename" => no_filename = true,
                "-H" | "--with-filename" => with_filename = true,
                "-o" | "--only-matching" => only_matching = true,
                "-q" | "--quiet" | "--silent" => quiet = true,
                "-r" | "-R" | "--recursive" => recursive = true,
                "-F" | "--fixed-strings" => fixed_strings = true,
                "-w" | "--word-regexp" => word_regexp = true,
                "-x" | "--line-regexp" => line_regexp = true,
                "-b" | "--byte-offset" => byte_offset = true,
                "--color" | "--colour" => color = true,
                "--no-color" | "--no-colour" => color = false,
                "-m" | "--max-count" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Option --max-count requires an argument".to_string());
                    }
                    max_count = Some(args[i].parse().map_err(|_| "Invalid max-count")?);
                }
                "-e" | "--regexp" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Option --regexp requires an argument".to_string());
                    }
                    pattern = args[i].clone();
                }
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                }
                _ => {
                    if pattern.is_empty() {
                        pattern = args[i].clone();
                    } else {
                        files.push(args[i].clone());
                    }
                }
            }
            i += 1;
        }

        if pattern.is_empty() {
            return Err("No pattern specified".to_string());
        }

        if files.is_empty() {
            files.push("-".to_string());
        }

        Ok(Config {
            pattern,
            files,
            ignore_case,
            invert_match,
            count,
            line_number,
            files_with_matches,
            files_without_match,
            no_filename,
            with_filename,
            only_matching,
            quiet,
            recursive,
            max_count,
            fixed_strings,
            word_regexp,
            line_regexp,
            color,
            byte_offset,
        })
    }
}

struct Matcher {
    regex: Regex,
    config: Config,
}

impl Matcher {
    fn new(config: Config) -> Result<Self, String> {
        let mut pattern = config.pattern.clone();

        if config.fixed_strings {
            pattern = regex::escape(&pattern);
        }

        if config.word_regexp {
            pattern = format!(r"\b{}\b", pattern);
        }

        if config.line_regexp {
            pattern = format!(r"^{}$", pattern);
        }

        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(config.ignore_case)
            .multi_line(true)
            .build()
            .map_err(|e| format!("Invalid regex: {}", e))?;

        Ok(Matcher { regex, config })
    }

    fn search_file(&self, path: &str) -> io::Result<bool> {
        if path == "-" {
            return self.search_stdin();
        }

        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        self.search_bytes(&mmap, path)
    }

    fn search_stdin(&self) -> io::Result<bool> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut found = false;
        let mut line_num = 0;
        let mut match_count = 0;
        let mut total_matches = 0;
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let mut lines: Vec<(usize, Vec<u8>)> = Vec::new();

        for line in reader.split(b'\n') {
            line_num += 1;
            let line_bytes = line?;
            lines.push((line_num, line_bytes));
        }

        for (idx, (num, line_bytes)) in lines.iter().enumerate() {
            if let Some(max) = self.config.max_count {
                if match_count >= max {
                    break;
                }
            }

            let is_match = self.regex.is_match(line_bytes);
            let should_print = is_match ^ self.config.invert_match;

            if should_print {
                found = true;
                match_count += 1;
                total_matches += 1;

                if self.config.quiet {
                    return Ok(true);
                }

                if self.config.files_with_matches {
                    writeln!(handle, "(standard input)")?;
                    return Ok(true);
                }

                if !self.config.count {
                    self.print_match(&mut handle, line_bytes, *num, None, idx, &lines)?;
                }
            }
        }

        if self.config.count {
            writeln!(handle, "{}", total_matches)?;
        }

        Ok(found)
    }

    fn search_bytes(&self, content: &[u8], filename: &str) -> io::Result<bool> {
        let mut found = false;
        let mut match_count = 0;
        let mut total_matches = 0;
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let lines: Vec<(usize, &[u8])> = content
            .split(|&b| b == b'\n')
            .enumerate()
            .map(|(i, line)| (i + 1, line))
            .collect();

        for (idx, (num, line)) in lines.iter().enumerate() {
            if let Some(max) = self.config.max_count {
                if match_count >= max {
                    break;
                }
            }

            let is_match = self.regex.is_match(line);
            let should_print = is_match ^ self.config.invert_match;

            if should_print {
                found = true;
                match_count += 1;
                total_matches += 1;

                if self.config.quiet {
                    return Ok(true);
                }

                if self.config.files_with_matches {
                    writeln!(handle, "{}", filename)?;
                    return Ok(true);
                }

                if !self.config.count {
                    self.print_match_bytes(&mut handle, line, *num, Some(filename), idx, &lines)?;
                }
            }
        }

        if self.config.count && !self.config.quiet {
            if self.config.with_filename
                || (!self.config.no_filename && self.config.files.len() > 1)
            {
                write!(handle, "{}:", filename)?;
            }
            writeln!(handle, "{}", total_matches)?;
        }

        if self.config.files_without_match && !found {
            writeln!(handle, "{}", filename)?;
        }

        Ok(found)
    }

    fn print_match<W: Write>(
        &self,
        handle: &mut W,
        line: &[u8],
        line_num: usize,
        filename: Option<&str>,
        idx: usize,
        all_lines: &[(usize, Vec<u8>)],
    ) -> io::Result<()> {
        if let Some(fname) = filename {
            if self.config.with_filename
                || (!self.config.no_filename && self.config.files.len() > 1)
            {
                if self.config.color {
                    write!(handle, "\x1b[35m{}\x1b[0m:", fname)?;
                } else {
                    write!(handle, "{}:", fname)?;
                }
            }
        }

        if self.config.line_number {
            if self.config.color {
                write!(handle, "\x1b[32m{}\x1b[0m:", line_num)?;
            } else {
                write!(handle, "{}:", line_num)?;
            }
        }

        if self.config.byte_offset {
            let mut offset = 0;
            for i in 0..idx {
                offset += all_lines[i].1.len() + 1;
            }
            write!(handle, "{}:", offset)?;
        }

        if self.config.only_matching && !self.config.invert_match {
            for mat in self.regex.find_iter(line) {
                if self.config.color {
                    write!(handle, "\x1b[1;31m")?;
                    handle.write_all(&line[mat.start()..mat.end()])?;
                    writeln!(handle, "\x1b[0m")?;
                } else {
                    handle.write_all(&line[mat.start()..mat.end()])?;
                    writeln!(handle)?;
                }
            }
        } else {
            if self.config.color && !self.config.invert_match {
                self.print_colored(handle, line)?;
            } else {
                handle.write_all(line)?;
                writeln!(handle)?;
            }
        }

        Ok(())
    }

    fn print_match_bytes<W: Write>(
        &self,
        handle: &mut W,
        line: &[u8],
        line_num: usize,
        filename: Option<&str>,
        idx: usize,
        all_lines: &[(usize, &[u8])],
    ) -> io::Result<()> {
        if let Some(fname) = filename {
            if self.config.with_filename
                || (!self.config.no_filename && self.config.files.len() > 1)
            {
                if self.config.color {
                    write!(handle, "\x1b[35m{}\x1b[0m:", fname)?;
                } else {
                    write!(handle, "{}:", fname)?;
                }
            }
        }

        if self.config.line_number {
            if self.config.color {
                write!(handle, "\x1b[32m{}\x1b[0m:", line_num)?;
            } else {
                write!(handle, "{}:", line_num)?;
            }
        }

        if self.config.byte_offset {
            let mut offset = 0;
            for i in 0..idx {
                offset += all_lines[i].1.len() + 1;
            }
            write!(handle, "{}:", offset)?;
        }

        if self.config.only_matching && !self.config.invert_match {
            for mat in self.regex.find_iter(line) {
                if self.config.color {
                    write!(handle, "\x1b[1;31m")?;
                    handle.write_all(&line[mat.start()..mat.end()])?;
                    writeln!(handle, "\x1b[0m")?;
                } else {
                    handle.write_all(&line[mat.start()..mat.end()])?;
                    writeln!(handle)?;
                }
            }
        } else {
            if self.config.color && !self.config.invert_match {
                self.print_colored(handle, line)?;
            } else {
                handle.write_all(line)?;
                writeln!(handle)?;
            }
        }

        Ok(())
    }

    fn print_colored<W: Write>(&self, handle: &mut W, line: &[u8]) -> io::Result<()> {
        let mut last = 0;
        for mat in self.regex.find_iter(line) {
            handle.write_all(&line[last..mat.start()])?;
            write!(handle, "\x1b[1;31m")?;
            handle.write_all(&line[mat.start()..mat.end()])?;
            write!(handle, "\x1b[0m")?;
            last = mat.end();
        }
        handle.write_all(&line[last..])?;
        writeln!(handle)?;
        Ok(())
    }

    fn search_directory(&self, path: &str) -> io::Result<bool> {
        let mut found = false;

        let entries: Vec<_> = std::fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

        for entry in entries {
            let path = entry.path();

            if path.is_dir() && self.config.recursive {
                if let Ok(result) = self.search_directory(path.to_str().unwrap()) {
                    found = found || result;
                }
            } else if path.is_file() {
                if let Ok(result) = self.search_file(path.to_str().unwrap()) {
                    found = found || result;
                }
            }
        }

        Ok(found)
    }
}

fn main() {
    let config = match Config::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Usage: rgrep [OPTIONS] PATTERN [FILE...]");
            eprintln!("\nOptions:");
            eprintln!("  -i, --ignore-case          Ignore case distinctions");
            eprintln!("  -v, --invert-match         Select non-matching lines");
            eprintln!("  -c, --count                Print only a count of matching lines");
            eprintln!("  -n, --line-number          Print line numbers");
            eprintln!("  -l, --files-with-matches   Print only names of files with matches");
            eprintln!("  -L, --files-without-match  Print only names of files without matches");
            eprintln!("  -h, --no-filename          Suppress file name prefix");
            eprintln!("  -H, --with-filename        Print file name for each match");
            eprintln!("  -o, --only-matching        Show only matching parts of lines");
            eprintln!("  -q, --quiet                Suppress all output");
            eprintln!("  -r, --recursive            Search directories recursively");
            eprintln!("  -F, --fixed-strings        Interpret pattern as fixed strings");
            eprintln!("  -w, --word-regexp          Match whole words only");
            eprintln!("  -x, --line-regexp          Match whole lines only");
            eprintln!("  -b, --byte-offset          Print byte offset of matches");
            eprintln!("  -m, --max-count NUM        Stop after NUM matches");
            eprintln!("      --color                Use colors in output");
            process::exit(2);
        }
    };

    let matcher = match Matcher::new(config) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(2);
        }
    };

    let mut found = false;

    for file in &matcher.config.files {
        let result = if Path::new(file).is_dir() {
            matcher.search_directory(file)
        } else {
            matcher.search_file(file)
        };

        match result {
            Ok(f) => found = found || f,
            Err(e) => {
                eprintln!("Error reading {}: {}", file, e);
            }
        }
    }

    process::exit(if found { 0 } else { 1 });
}
