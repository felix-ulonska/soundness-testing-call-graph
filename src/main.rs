pub mod valgrind_parser;
pub mod valgrind;
pub mod cwe_checker;
mod soudness_test;
use std::{ fs, path::{Path, PathBuf}};
use chrono::Local;
use cwe_checker::{complete_analysis, setup_hetzner_server};
use soudness_test::soundness;
use valgrind::run_valgrind;

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    binary: PathBuf,
    //cwe_checker_results: PathBuf,
    ///// Use if the binary is small
    //run_cwe_checker: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    //gawk: let bin_to_analyis = PathBuf::from("/nix/store/3p3fwczck2yn1wwfjnymzkz8w11vbvg7-gawk-5.3.1/bin/gawk");
    let bin_to_analyis = PathBuf::from(args.binary);
    //setup_hetzner_server(bin_to_analyis);
    let cwe_checker_results = complete_analysis(&bin_to_analyis);
    //let cwe_checker_results = parse_results_file(&bin_to_analyis);
    let output_folder = Path::new("output");
    let date = Local::now();
    let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d-%H-%M-%S").to_string()));

    let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));
    fs::create_dir_all(output_folder).unwrap();

    let valgrind_result = run_valgrind(&bin_to_analyis, &valgrind_output_file).await;
    soundness(&cwe_checker_results, &valgrind_result);
}




