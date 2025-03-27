

fn main() {
    let output_folder = "output";
    let output_folder = format!("./output/{}", date.format("%Y-%m-%d][%H-%M-%S"));

    run_valgrind("/home/jabbi/Projects/masterarbeit/test_targets/stack_tests/global_func_ptr", output_folder);
}

fn run_valgrind(binary: &str, output_file: &str) {
    Command::new("valgrind")
        .args(["--tool=callgrind", "--dump-instr=yes", format!("--callgrind-out-file={}", output_file), "--collect-jumps=yes", binary])
        .expect("failed to execute process")
}
