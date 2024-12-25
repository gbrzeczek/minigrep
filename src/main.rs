use colored::Colorize;
use std::{env, fs, io::{self, BufRead}, process};

use regex::Regex;

struct ColorableSlice<'a> {
    slice: &'a str,
    should_color: bool,
}

const USAGE_INSTRUCTION: &str = "Usage: minigrep [file path] [pattern]";

fn main() {
    let args: Vec<String> = env::args().collect();

    let run = match args.len() {
        1 => panic!("TODO: this should be a print and process exit"),
        2 => run_from_stdin(args),
        _ => run_from_file(args)
    };

    if let Err(e) = run {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run_from_file(args: Vec<String>) -> Result<(), String> {

    let file_path = args
        .get(1)
        .ok_or(format!("File path argument is missing.\n\n{}", USAGE_INSTRUCTION))?;
    let pattern_string = args
        .get(2)
        .ok_or(format!("Pattern argument is missing.\n\n{}", USAGE_INSTRUCTION))?;

    let absolute_path = std::fs::canonicalize(file_path)
        .map_err(|err| format!("Failed to resolve path '{}': {}", &file_path, err))?;

    let pattern = Regex::new(pattern_string).expect(USAGE_INSTRUCTION);

    let lines: Vec<_> = fs::read_to_string(&absolute_path)
        .map_err(|err| format!("Failed to read file {}: {}", &absolute_path.display(), err))?
        .lines()
        .map(|l| String::from(l.trim()))
        .collect();

    filter_and_print(lines, &pattern);

    Ok(())
}

fn run_from_stdin(args: Vec<String>) -> Result<(), String> {
    let pattern_string = args
        .get(1)
        .ok_or(format!("Pattern argument is missing.\n\n{}", USAGE_INSTRUCTION))?;

    let pattern = Regex::new(pattern_string).expect(USAGE_INSTRUCTION);

    let lines = io::stdin()
        .lock()
        .lines()
        .map(|l| l.map_err(|e| format!("Error reading input: {}", e)))
        .collect::<Result<Vec<String>, String>>()?;

    filter_and_print(lines, &pattern);
    
    Ok(())
}

fn filter_and_print(lines: Vec<String>, pattern: &Regex) {
    let lines: Vec<_> = lines
        .iter()
        .filter_map(|l| to_colorable_slices(l, pattern))
        .collect();

    for slices in lines {
        for slice in slices {
            if slice.should_color {
                print!("{}", slice.slice.green().underline());
            } else {
                print!("{}", slice.slice);
            }
        }

        println!();
    }
}

fn to_colorable_slices<'a>(line: &'a str, pattern: &Regex) -> Option<Vec<ColorableSlice<'a>>> {
    if line.is_empty() {
        return None;
    }

    let captures = pattern.find_iter(line).collect::<Vec<_>>();

    if captures.is_empty() {
        return None;
    }

    let positions: Vec<usize> = captures.iter().flat_map(|m| [m.start(), m.end()]).collect();

    let len = line.len();

    let positions = std::iter::once(&0)
        .chain(positions.iter())
        .chain(std::iter::once(&len))
        .collect::<Vec<_>>();

    let slices = positions
        .windows(2)
        .enumerate()
        .filter_map(|(i, window)| {
            let start = *window[0];
            let end = *window[1];

            if start != end {
                let slice = ColorableSlice {
                    slice: &line[start..end],
                    should_color: i % 2 != 0,
                };
                Some(slice)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Some(slices)
}
