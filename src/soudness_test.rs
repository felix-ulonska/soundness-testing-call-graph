use crate::{cwe_checker::CweCheckerResult, valgrind::ValgrindResult};


pub fn soundness(cwe_checker: &CweCheckerResult, real: &ValgrindResult) {
    let mut is_unsound = false;
    // Only Indirect calls from the program
    let mut real_calls = real.calls.clone();
    // We sort to have a function, by function analysis. It just nicer to read
    real_calls.sort_by_key(|call| call.from_instr);
    let real_calls_from_prog_region = real.calls.iter().filter(|call| cwe_checker.metadata.indirect_call_sites.contains(&call.from_instr));

    let mut current_function = -1;
    for call in real_calls_from_prog_region {
        if current_function != call.in_fn {
            println!("=== Function: {:10} ===", real.get_function_of_call(call));
            current_function = call.in_fn;
        }

        if call.does_jump_object_file {
            println!("\tCall {} between object files. Ignoring", call);
            continue;
        }

        let Some(callsite) = cwe_checker.get_call_site(call.from_instr) else {
            println!("\tCallsite {} is missing", call);
            is_unsound = true;
            continue;
        };
        if callsite.has_target(&call.to_instr) {
            println!("\t{:#x} -> {:#x} @ {}", call.from_instr, call.to_instr, real.get_target_function_of_call(call));
        } else {
            is_unsound = true;
            println!("\t[!] UNSOUND: {:#x} -> {:#x}", call.from_instr, call.to_instr);
        }
    }

    if is_unsound {
        println!("IS_UNSOUND");
    } else {
        println!("IS_SOUND");
    }
}
