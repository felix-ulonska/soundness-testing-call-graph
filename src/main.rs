pub mod valgrind_parser;
use std::{collections::HashMap, fmt::Display, fs, os::unix::process::CommandExt, path::{Path, PathBuf}, process::Command};
use chrono::Local;
use valgrind_parser::{parse_valgrind_file, InstrCounter, PositionName};

fn main() {
    let output_folder = Path::new("output");
    let date = Local::now();
    let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d-%H-%M-%S").to_string()));

    let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));
    fs::create_dir_all(output_folder).unwrap();

    run_valgrind("/home/jabbi/Projects/masterarbeit/test_targets/stack_tests/global_func_ptr", &valgrind_output_file);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Call {
    from_instr: i64,
    to_instr: i64,
    in_fn: i64
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {} @ {}", self.from_instr, self.to_instr, self.in_fn)
    }
}

fn run_valgrind(binary: &str, output_file: &PathBuf) {
    let output_file_arg = format!("--callgrind-out-file={}", output_file.to_str().unwrap());
    let output = Command::new("valgrind")
        .args(["--tool=callgrind", "--dump-instr=yes", &output_file_arg, "--collect-jumps=yes", binary])
        .output();
    output.expect("Valgrind failed");

    let content = fs::read_to_string(output_file).unwrap();
    let valgrind_file = parse_valgrind_file(&content);
    let mut valgrind_name_cache = ValgrindNameCache::new();
    let mut curr_index = 0;
    let mut curr_fn_index = 0;

    let mut calls = vec![];
    for line in valgrind_file.unwrap().1 {
        match line {
            valgrind_parser::ValgrindLine::FnLine(fn_) => {
                valgrind_name_cache.add(&fn_);
                curr_fn_index = fn_.number.unwrap();
            }
            valgrind_parser::ValgrindLine::CfnLine(cfn) => {
                valgrind_name_cache.add(&cfn.position_name);
                calls.push(Call {
                    from_instr: curr_index,
                    to_instr: match cfn.instr {
                        InstrCounter::Absolute(abs) => abs,
                        InstrCounter::Relative(relative) => curr_index + relative,
                        InstrCounter::Same() => curr_index
                    },
                    in_fn: curr_fn_index
                });
            },
            valgrind_parser::ValgrindLine::InstrCounter(valgrind_parser::InstrCounter::Absolute(new_index)) => {curr_index = new_index},
            _ => ()
        }
    }

    for call in calls {
        println!("{} -> {} @ {}", call.from_instr, call.to_instr, valgrind_name_cache.get(call.in_fn));
    }
}

struct ValgrindNameCache {
    name_cache: HashMap::<i64, String>
}

impl ValgrindNameCache {
    fn new() -> ValgrindNameCache {
        ValgrindNameCache {
            name_cache: HashMap::new()
        }
    }

    fn add(&mut self, pos: &PositionName) {
        if let PositionName {number: Some(number), trailing: Some(name)} = pos {
            self.name_cache.insert(number.clone(), name.to_string());
        }
    }

    fn get(&self, index: i64) -> String {
        self.name_cache.get(&index).unwrap_or(&index.to_string()).to_string()
    }
}
