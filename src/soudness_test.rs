use crate::{
    cwe_checker::CweCheckerResult,
    valgrind::{RealCall, ValgrindResult},
};

fn offset_based_on_main(cwe_checker: &CweCheckerResult, real: &ValgrindResult) -> i64 {
    let Some(main_function) = cwe_checker.metadata.functions.iter().find(|function| function.name == "main") else { return 0; };
    let cwe_function_offset = main_function.address;

    // find main
    let Some(main_address_real) = real.valgrind_name_cache.name_cache.iter().find_map(|(addr, name)| {if name == "main" { Some(addr) } else {None}}) else {return 0;};

    (*main_address_real - cwe_function_offset) as i64
}

pub fn soundness(cwe_checker: &CweCheckerResult, real: &ValgrindResult) {
    let mut is_unsound = false;
    // Only Indirect calls from the program
    let mut real_calls = real.calls.clone();
    // We sort to have a function, by function analysis. It just nicer to read
    real_calls.sort_by_key(|call| call.from_instr);

    let offset = offset_based_on_main(cwe_checker, real);

    println!(
        "indirect_call_sites {:?}",
        cwe_checker
            .metadata
            .indirect_call_sites
            .iter()
            .map(|target| format!("{:x}", target))
            .collect::<Vec<String>>()
            .join(",")
    );
    let real_calls_from_prog_region = real
        .calls
        .iter()
        .filter(|call| {
            cwe_checker
                .metadata
                .indirect_call_sites
                .contains(&call.from_instr)
        })
        .collect::<Vec<&RealCall>>();

    let mut current_function = -1;
    for call in real_calls_from_prog_region {
        if current_function != call.in_fn {
            println!("=== Function: {:10} ===", real.get_function_of_call(&call));
            current_function = call.in_fn;
        }

        if call.does_jump_object_file {
            println!("\tCall {} between object files. Ignoring", call);
            continue;
        }

        let Some(callsite) = cwe_checker.get_call_site(((call.from_instr as i64) - offset) as u64) else {
            println!("\tCallsite {} is missing", call);
            is_unsound = true;
            continue;
        };
        if callsite.has_target(&(((call.to_instr as i64) - offset) as u64)) {
            println!(
                "\t{:#x} -> {:#x} @ {}",
                call.from_instr,
                call.to_instr,
                real.get_target_function_of_call(&call)
            );
        } else {
            is_unsound = true;
            println!(
                "\t[!] UNSOUND: {:#x} -> {:#x}",
                call.from_instr, call.to_instr
            );
        }
    }

    if is_unsound {
        println!("IS_UNSOUND");
    } else {
        println!("IS_SOUND");
    }
}
