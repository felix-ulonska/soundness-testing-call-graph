pub mod valgrind_parser;
pub mod valgrind;
pub mod cwe_checker;
use std::{ fs, path::{Path, PathBuf}};
use chrono::Local;
use cwe_checker::setup_hetzner_server;
use valgrind::run_valgrind;

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    binary: PathBuf,
    cwe_checker_results: PathBuf,
    /// Use if the binary is small
    run_cwe_checker: bool,
}

fn main() {
    setup_hetzner_server(PathBuf::from("/home/jabbi/Projects/masterarbeit/test_targets/stack_tests/global_func_ptr"));
    return;
    let output_folder = Path::new("output");
    let date = Local::now();
    let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d-%H-%M-%S").to_string()));

    let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));
    fs::create_dir_all(output_folder).unwrap();

    run_valgrind("/home/jabbi/Projects/masterarbeit/test_targets/stack_tests/global_func_ptr", &valgrind_output_file);
}


