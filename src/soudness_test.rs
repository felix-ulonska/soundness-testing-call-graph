use crate::{
    cwe_checker::CweCheckerResult,
    valgrind::{RealCall, ValgrindResult},
};

fn offset_based_on_main(cwe_checker: &CweCheckerResult, real: &ValgrindResult) -> i64 {
    let Some(main_function) = cwe_checker.metadata.functions.iter().find(|function| function.name == "main") else { return 0; };
    let cwe_function_offset = main_function.address;

    // find main
    let Some(fn_id) = real.valgrind_name_cache.name_cache.iter().find_map(|(fn_id, name)| {if name == "main" { Some(fn_id) } else {None}}) else {return 0;};

    let main_address_real = real.base_address_mapping.get(fn_id).unwrap();
    println!("Main: {}, cwe_function_offset: {}, cal {}", *main_address_real, cwe_function_offset, (*main_address_real - cwe_function_offset) as i64);
    (*main_address_real - cwe_function_offset) as i64
}

pub struct SoundnessReport {
    pub checked_calls: i64,
    pub sound_calls: i64,
}

impl SoundnessReport {
    pub fn to_csvline(self: Self) -> String {
        format!("self.sound_calls")
    }
}

pub fn soundness(cwe_checker: &CweCheckerResult, real: &ValgrindResult) -> SoundnessReport {
    let mut soundness_report = SoundnessReport { checked_calls: 0, sound_calls: 0 };
    let mut is_unsound = false;
    // Only Indirect calls from the program
    let mut real_calls = real.calls.clone();
    // We sort to have a function, by function analysis. It just nicer to read
    real_calls.sort_by_key(|call| call.from_instr);

    let offset = offset_based_on_main(cwe_checker, real);
    println!("Using offset: {}", offset);

    let mut sorted_funcs = cwe_checker.metadata.functions.clone();
    sorted_funcs.sort_by_key(|func| func.address);
    sorted_funcs.reverse();
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
                .contains(&(((call.from_instr as i64) - offset) as u64))
        })
        .collect::<Vec<&RealCall>>();

    let mut current_function_cwe = "".to_string();
    for call in real_calls_from_prog_region {
        for func in &sorted_funcs {
            if (func.address) < ((call.from_instr as i64) - offset) as u64 {
                current_function_cwe = func.name.clone();
                break
            }

        }

        if current_function_cwe == "__libc_csu_init" {
            continue;
        }

        if call.does_jump_object_file {
            println!("\tCall {} between object files. Ignoring", call);
            continue;
        }
        soundness_report.checked_calls += 1;

        println!("Using from_inst {}", ((call.from_instr as i64) - offset) as u64);
        let Some(callsite) = cwe_checker.get_call_site(((call.from_instr as i64) - offset) as u64) else {
            println!("\tCallsite {} is missing", call);
            is_unsound = true;
            continue;
        };
        if callsite.has_target(&(((call.to_instr as i64) - offset) as u64)) {
            soundness_report.sound_calls += 1;
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

    soundness_report
}
