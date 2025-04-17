use std::{collections::HashMap, path::PathBuf};

use nom::{bytes::tag, number::complete::hex_u32, sequence::preceded, IResult, Parser};

use crate::valgrind::{RealCall, ValgrindNameCache, ValgrindResult};

#[derive(Debug, serde::Deserialize)]
struct Record {
    from_instr: i64,
    to_instr: i64,
    binary_name: String,
}

//#[derive(Clone, Debug, PartialEq, Eq)]
//pub struct RealCall {
//    pub from_instr: u64,
//    pub to_instr: u64,
//    pub in_fn: i64,
//    pub target_fn: i64,
//    pub does_jump_object_file: bool,
//}
//
//impl Display for RealCall {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{:#x} -> {:#x} @ {}", self.from_instr, self.to_instr, self.in_fn)
//    }
//}
fn parse_hex(input: &str) -> IResult<&str, u64> {
    preceded(tag("0x"), hex_u32)
        .parse(input)
        .and_then(|result| Ok((result.0, result.1 as u64)))
}

pub fn load_callee_from_csv(path: &PathBuf, binary_name: &str) -> ValgrindResult {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();

    let mut calls = vec![];

    for result in rdr.records() {
        let record = result.unwrap();
        let (_, from_instr): (&str, u64) =
            parse_hex(record.get(0).unwrap()).expect("Failed parsing hex");
        let (_, to_instr): (&str, u64) =
            parse_hex(record.get(1).unwrap()).expect("Failed parsing hex");
        let src_obj = record.get(2).unwrap();

        if binary_name != src_obj {
            continue;
        }

        let new_call = RealCall {
            from_instr: (from_instr + 0x400000),
            to_instr: (to_instr + 0x400000),
            does_jump_object_file: false,
            in_fn: 0,
            target_fn: 0,
        };
        calls.push(new_call);
    }

    ValgrindResult {
        calls,
        valgrind_name_cache: ValgrindNameCache::new(),
        base_address_mapping: HashMap::new()
    }
}
