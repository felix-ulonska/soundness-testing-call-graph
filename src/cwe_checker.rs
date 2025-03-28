use std::{fs::{self, File}, io::{BufRead, BufReader, Read, Write}, path::PathBuf, process::{Command, Stdio}};

use serde::{Deserialize, Serialize};

// Copy from json_export.rs in cwe_checker

#[derive(Serialize, Deserialize, Debug)]
pub struct Call {
    pub from_instr: u64,
    pub to_instr: u64,
    pub is_indirect: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub address_base_offset: u64,
    pub indirect_call_sites: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportCallGraph {
    pub metadata: Metadata,
    pub calls: Vec<Call>,
}

fn get_analysis_results(report: PathBuf) {
    let content = fs::read_to_string(report).unwrap();
    let callgraph: ExportCallGraph = serde_json::from_str(&content).expect("JSON is bad");
    println!("{:#?}", callgraph);
}

pub fn run_cwe_checker(binary: &PathBuf, output_file: &PathBuf) {
    let output = Command::new("cwe-checker")
        .args([binary])
        .output().expect("Failed exeucting cwe_checker");

    let json = output.stdout;
    let mut file = File::create(output_file).expect("Could not create output file");
    file.write_all(&json).expect("Could not write json to file");
}

fn start_analysis(should_run_cwe_checker: bool, binary: PathBuf) {
    let output_file = PathBuf::from("output_cwe_checker.json");
    if should_run_cwe_checker {
        run_cwe_checker(&binary, &output_file);
    }
}

pub fn setup_hetzner_server(binary: PathBuf) {
    println!("You want to run the binary remote?");
    println!("Setup a $Cloud Service with a common distro (a lot of cores and ram) and run the following command");
    println!("curl -sSf -L https://install.lix.systems/lix | sh -s -- install --enable-flakes --no-confirm");
    println!(". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh");
    println!("curl https://getcroc.schollz.com | bash");

    let mut child = Command::new("croc")
        .arg("--yes")
        .arg("send")
        .arg(binary.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start croc process");

    if let Some(stdout) = &mut child.stderr {
        let reader = BufReader::new(stdout);
        let mut next_line_croc = false; 
        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            if next_line_croc {
                print!("{}", line.trim()); // This will print croc code and status updates
                println!(" --yes");
                next_line_croc = false;

                let file_name_str = binary.file_name().unwrap().to_str().unwrap();
                println!("nix run github:felix-ulonska/cwe_checker/feat/split_stack_mem --extra-experimental-features flakes --option tarball-ttl 0 -- {} > {}_call_graph.json", file_name_str, file_name_str);
            }
            if line.contains("For Linux") {
                next_line_croc = true;
            }
        }
    }

    println!("Received on server!");
    println!("Afterwards, you can use croc to send the file back");
    println!("DO NOT FORGET to stop the server; otherwise the bill will be heavy");
}
