pub mod valgrind_parser;
pub mod valgrind;
pub mod cwe_checker;
pub mod load_from_callee_csv;
mod soudness_test;
use std::{ fs, path::{Path, PathBuf}};
use chrono::Local;
use cwe_checker::{complete_analysis, get_analysis_results, setup_hetzner_server};
use nix::sys::wait::wait;
use soudness_test::soundness;
use valgrind::run_valgrind;

use clap::{arg, Parser, Subcommand};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// Subcommands for different tasks
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Subcommand to run the CWE checker
    ShowHetznerHelp {
        /// Path to the binary to check
        binary_path: String,
        
    },

    /// Subcommand to use callee 
    SoundnessTest {
        /// Path to the binary to check
        #[arg(long)]
        binary_path: Option<PathBuf>,
        /// Run the CWE checker (default: true)
        #[arg(long)]
        cwe_checker_result: Option<PathBuf>,

        #[arg(long)]
        callee_csv: Option<PathBuf>,
        #[arg(long)]
        callee_bin_name: Option<String>
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::ShowHetznerHelp { binary_path } => {
            let bin_to_analyis = PathBuf::from(binary_path);
            setup_hetzner_server(bin_to_analyis);
        },
        Commands::SoundnessTest { binary_path, cwe_checker_result, callee_csv, callee_bin_name } => {
            let cwe_checker_results = match cwe_checker_result {
                Some(path) => get_analysis_results(&path),
                None => complete_analysis(&binary_path.clone().expect("If cwe_checker_result is not set, the bianry needs to be set")),
            };
            let valgrind_result = match callee_csv {
                Some(path) => {load_from_callee_csv::load_callee_from_csv(&path, &callee_bin_name.expect("If using callee_csv, callee_bin_name needs to be set") )},
                None => {
                    let output_folder = Path::new("output");
                    let date = Local::now();
                    let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d-%H-%M-%S").to_string()));

                    let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));
                    fs::create_dir_all(output_folder).unwrap();

                    let valgrind_result = run_valgrind(&binary_path.expect("If callee_csv is not set, the bianry needs to be set"), &valgrind_output_file).await;
                    valgrind_result
                }
            };

            soundness(&cwe_checker_results, &valgrind_result);
        },
    };
}

