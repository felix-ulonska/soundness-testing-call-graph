use std::{collections::{HashSet, VecDeque}, fs, os::unix::process::CommandExt, path::{Path, PathBuf}, process::Command};
use chrono::Local;
use regex::{Captures, Regex};

fn main() {
    let output_folder = Path::new("output");
    let date = Local::now();
    let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d][%H-%M-%S").to_string()));

    let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));

    run_valgrind("/home/jabbi/Projects/masterarbeit/test_targets/stack_tests/global_func_ptr", &valgrind_output_file);
}

fn run_valgrind(binary: &str, output_file: &PathBuf) {
    let output_file_arg = format!("--callgrind-out-file={}", output_file.to_str().unwrap());
    Command::new("valgrind")
        .args(["--tool=callgrind", "--dump-instr=yes", &output_file_arg, "--collect-jumps=yes", binary])
        .exec();

    let content = fs::read_to_string(output_file);
}

struct ValgrindParser {
    fn_name_map: HashSet::<i32, String>;
    ob_name_map: HashSet::<i32, String>;
}

impl ValgrindParser {
    fn new() -> ValgrindParser {
        ValgrindParser {
            fn_name_map: HashSet::new(),
            ob_name_map: HashSet::new()
        }
    }

    fn parse_valgrind(&mut self, content: &str) {
        let mut current_instr_counter: i32 = 0;
        let mut lines = content.split("\n").collect::<VecDeque<&str>>();
        let mut prelude_done = false;
        while !lines.is_empty() {
            let line = lines.pop_front().unwrap().trim();
            if line.starts_with("summary:") {
                prelude_done = true;
                continue;
            }

            if !prelude_done {
                continue;
            }

            let command = parse_command(line);

            match command {
                Some(_) => todo!(),
                None => {

                }
            }

            let cost_line = parse_costline(line);
            match cost_line {
                Some(InstrCounter::Absolute(abs)) => {current_instr_counter = abs}
                None | Some(InstrCounter::Same()) | Some(InstrCounter::Relative(_)) => (),
            }
        }
    }
}


enum Assigment {
    Ob(String),
    Fn(String),
    Unknown
}

enum InstrCounter {
    Absolute(i32),
    Relative(i32),
    Same()
}

// CostLine := SubPositionList Costs?
// SubPositionList := (SubPosition+ Space+)+
// SubPosition := Number | "+" Number | "-" Number | "*"
// https://regex101.com/r/fhByTC/1
fn parse_costline(cost_line: &str) -> Option<InstrCounter> {
    let pattern = r"^((0x[[:xdigit:]]+)|(\+[[:digit:]]+)|(-[[:digit:]]+)|(\*))\s+[[:digit:]]+.*$";

    let re = Regex::new(pattern).unwrap();
    let Some(captures) = re.captures(cost_line) else {return None;};

    if let Some(const_addr) = captures.get(1) {
        Some(InstrCounter::Absolute(const_addr.as_str().parse::<i32>().unwrap()))
    } else if let Some(plus) = captures.get(2) {
        Some(InstrCounter::Relative(plus.as_str().trim_start_matches('+').parse::<i32>().unwrap()))
    } else if let Some(minus) = captures.get(3) {
        Some(InstrCounter::Relative(-minus.as_str().trim_start_matches('-').parse::<i32>().unwrap()))
    } else if let Some(_) = captures.get(4) {
        Some(InstrCounter::Same())
    } else {
        panic!("HOW DID THIS HAPPEN?")
    }
}

// parse position_name
fn parse_shortname(shortname: &str) {

}

fn parse_command(line: &str) -> Option<Assigment> {
    match line.split_once("=") {
        Some(("ob", object)) => Some(Assigment::Ob(object.to_string())),
        Some(("fn", function)) => {
            Some(Assigment::Ob(function.to_string()))
        },
        Some(_) => Some(Assigment::Unknown),
        None => None,
    }
}
