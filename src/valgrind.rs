use std::{collections::HashMap, fmt::Display, fs, path::PathBuf, process::Command};
use crate::valgrind_parser::{parse_valgrind_file, InstrCounter, PositionName, ValgrindLine};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealCall {
    pub from_instr: u64,
    pub to_instr: u64,
    pub in_fn: i64,
    pub target_fn: i64,
    pub does_jump_object_file: bool,
}

impl Display for RealCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x} -> {:#x} @ {}", self.from_instr, self.to_instr, self.in_fn)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValgrindResult {
    pub calls: Vec<RealCall>,
    valgrind_name_cache: ValgrindNameCache
}

impl ValgrindResult {
    pub fn get_function_of_call(&self, call: &RealCall) -> String {
        self.valgrind_name_cache.get(call.in_fn.try_into().unwrap())
    }
    pub fn get_target_function_of_call(&self, call: &RealCall) -> String {
        self.valgrind_name_cache.get(call.target_fn.try_into().unwrap())
    }
}

pub fn run_valgrind(binary: &PathBuf, output_file: &PathBuf) -> ValgrindResult {
    let output_file_arg = format!("--callgrind-out-file={}", output_file.to_str().unwrap());
    let output = Command::new("valgrind")
        .args(["--tool=callgrind", "--dump-instr=yes", &output_file_arg, "--collect-jumps=yes", binary.to_str().unwrap()])
        .output();
    output.expect("Valgrind failed");

    let content = fs::read_to_string(output_file).unwrap();
    let valgrind_file = parse_valgrind_file(&content);
    let mut valgrind_name_cache = ValgrindNameCache::new();
    let mut curr_index: u64 = 0;
    let mut curr_fn_index = 0;

    let mut calls = vec![];
    for line in valgrind_file.unwrap().1 {
        match line {
            ValgrindLine::FnLine(fn_) => {
                valgrind_name_cache.add(&fn_);
                curr_fn_index = fn_.number.unwrap();
            }
            ValgrindLine::CfnLine(cfn) => {
                valgrind_name_cache.add(&cfn.position_name);
                calls.push(RealCall {
                    from_instr: match cfn.from_instr {
                        InstrCounter::Absolute(abs) => {curr_index = abs; abs as u64},
                        InstrCounter::Relative(relative) => {curr_index = (curr_index as i64  + relative) as u64; curr_index},
                        InstrCounter::Same() => curr_index as u64
                    },
                    to_instr: match cfn.target_instr {
                        InstrCounter::Absolute(abs) => {abs as u64},
                        InstrCounter::Relative(relative) => (curr_index as i64 + relative) as u64,
                        InstrCounter::Same() => curr_index as u64
                    },
                    in_fn: curr_fn_index as i64,
                    target_fn: cfn.position_name.number.unwrap().try_into().unwrap(),
                    does_jump_object_file: cfn.next_object_file.is_some()
                });
            },
            ValgrindLine::InstrCounter(InstrCounter::Absolute(new_index)) => {curr_index = new_index},
            ValgrindLine::InstrCounter(InstrCounter::Relative(relative)) => {curr_index = (curr_index as i64  + relative) as u64;},
            _ => ()
        }
    }

    //for call in &calls {
    //    println!("{} -> {} @ {}", call.from_instr, call.to_instr, valgrind_name_cache.get(call.in_fn.try_into().unwrap()));
    //}
    //
    ValgrindResult {
        calls, valgrind_name_cache
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct ValgrindNameCache {
    name_cache: HashMap::<u64, String>
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

    fn get(&self, index: u64) -> String {
        self.name_cache.get(&index).unwrap_or(&index.to_string()).to_string()
    }
}
