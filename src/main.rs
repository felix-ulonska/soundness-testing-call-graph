pub mod valgrind_parser;
pub mod valgrind;
pub mod cwe_checker;
pub mod load_from_callee_csv;
mod soudness_test;
use std::{ fs, path::{Path, PathBuf}};
use chrono::Local;
use cwe_checker::{complete_analysis, get_analysis_results, setup_hetzner_server};
use load_from_callee_csv::load_callee_from_csv;
use soudness_test::soundness;
use valgrind::{analyze_valgrind, run_valgrind};

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
        valgrind_output: Option<PathBuf>,

        #[arg(long)]
        callee_csv: Option<PathBuf>,
        #[arg(long)]
        callee_bin_name: Option<String>
    },

    ProcessCalleeResults {
        #[arg(long)]
        cwe_checker_result_folder: PathBuf,

        #[arg(long)]
        callee_csv: PathBuf,
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::ShowHetznerHelp { binary_path } => {
            let bin_to_analyis = PathBuf::from(binary_path);
            setup_hetzner_server(bin_to_analyis);
        },
        Commands::SoundnessTest { binary_path, cwe_checker_result, callee_csv, callee_bin_name, valgrind_output } => {
            let cwe_checker_results = match cwe_checker_result {
                Some(path) => get_analysis_results(&path).unwrap(),
                None => complete_analysis(&binary_path.clone().expect("If cwe_checker_result is not set, the bianry needs to be set")),
            };
            let valgrind_result = match callee_csv {
                Some(path) => {load_from_callee_csv::load_callee_from_csv(&path, &callee_bin_name.expect("If using callee_csv, callee_bin_name needs to be set") )},
                None => {
                    match valgrind_output {
                        Some(valgrind_output) => analyze_valgrind(&valgrind_output),
                        None => {
                            let output_folder = Path::new("output");
                            let date = Local::now();
                            let output_folder = output_folder.join(Path::new(&date.format("%Y-%m-%d-%H-%M-%S").to_string()));

                            let valgrind_output_file = output_folder.join(Path::new("valgrind.out"));
                            fs::create_dir_all(output_folder).unwrap();

                            let valgrind_result = run_valgrind(&binary_path.expect("If callee_csv is not set, the bianry needs to be set"), &valgrind_output_file).await;
                            valgrind_result
                            }
                        }
                }
            };
            soundness(&cwe_checker_results, &valgrind_result);
        },
        Commands::ProcessCalleeResults { cwe_checker_result_folder, callee_csv } => {
            let paths = fs::read_dir(cwe_checker_result_folder).unwrap();
            let mut csv = "".to_owned();
            for path in paths {
                let path = path.unwrap();
                let file_name = path.file_name().to_str().clone().unwrap().split(".").collect::<Vec<&str>>()[0].to_owned();
                println!("{}", file_name);
                let file_splited = file_name.split("_").collect::<Vec<&str>>();
                if file_splited.len() != 3 {
                    continue;
                }
                let binary = file_splited[0];
                let compiler = file_splited[1];
                let opt = file_splited[2];

                let Some(analysis_result) = get_analysis_results(&path.path()) else {
                    println!("For {} the analysis was not successful", binary);
                    continue
                };

                let real = load_callee_from_csv(&callee_csv, &format!("{}-{}-{}", compiler, opt, binary));
                let soundness_report = soundness(&analysis_result, &real);
                csv = format!("{}\n{},{},{},{}", csv, file_name, soundness_report.checked_calls, soundness_report.sound_calls, soundness_report.sound_calls as f32 / soundness_report.checked_calls as f32);
            }
            println!("{}", csv);
        },
    };
}

