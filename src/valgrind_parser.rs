use nom::{bytes::complete::{tag, take_until}, character::complete::{u64 as parseu64, line_ending, space0, space1}, combinator::{map, opt}, multi::many0, number::complete::hex_u32, sequence::{preceded, terminated}, IResult, Parser};
use nom::sequence::delimited;


fn parse_paren_number(input: &str) -> IResult<&str, u64> {
    delimited(tag("("), parseu64, tag(")")).parse(input)
}

fn parse_no_newline_chars(input: &str) -> IResult<&str, &str> {
    take_until("\n").parse(input)
}
fn parse_till_eol(input: &str) -> IResult<&str, &str> {
    terminated(
        take_until("\n"),
        line_ending,
    ).parse(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionName {
    pub number: Option<u64>,
    pub trailing: Option<String>,
}

fn parse_fn_line(input: &str) -> IResult<&str, PositionName> {
    delimited(tag("fn="), parse_position_name, line_ending).parse(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CfnLine {
    pub position_name: PositionName,
    pub target_instr: InstrCounter,
    pub from_instr: InstrCounter,
}

/// looks like this
/// cfn=(122) _dl_map_object
/// calls=1 0x7b20 0 
/// * 0 2935
fn parse_cfn(input: &str) -> IResult<&str, CfnLine> {
    let (input, position_name) = delimited(tag("cfn="), parse_position_name, line_ending).parse(input)?;
    let (input, target_instr) = parse_calls_line(input)?;
    let (input, from_instr) = parse_costline(input)?;

    Ok((input, CfnLine {
        position_name,
        target_instr,
        from_instr
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValgrindLine {
    FnLine(PositionName),
    CfnLine(CfnLine),
    InstrCounter(InstrCounter)
} 

fn parse_line(input: &str) -> IResult<&str, Option<ValgrindLine>> {
    nom::branch::alt((
        map(parse_costline, |val| Some(ValgrindLine::InstrCounter(val))),
        map(parse_fn_line, |val| Some(ValgrindLine::FnLine(val))),
        map(parse_cfn, |val| Some(ValgrindLine::CfnLine(val))),
        map(parse_till_eol, |_| None)
    )).parse(input)
}

pub fn parse_valgrind_file(input: &str) -> IResult<&str, Vec<ValgrindLine>> {
    let (input, output) = many0(parse_line).parse(input)?;
    Ok((input, output.iter().filter_map(|line| line.clone()).collect::<Vec<ValgrindLine>>()))
}

pub fn parse_position_name(input: &str) -> IResult<&str, PositionName> {
    let (input, number) = opt(parse_paren_number).parse(input)?;
    let (input, trailing) = opt(
        preceded(space1, parse_no_newline_chars)
    ).parse(input)?;

    Ok((input, PositionName { number, trailing: trailing.and_then(|val| Some(val.to_string())) }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstrCounter {
    Absolute(u64),
    Relative(i64),
    Same()
}

fn parse_subposition(input: &str) -> IResult<&str, InstrCounter> {
    let plus = preceded(tag("+"), parseu64);
    let minus = preceded(tag("-"), parseu64);
    let absolute = preceded(tag("0x"), hex_u32);
    let star = map(tag("*"), |_| InstrCounter::Same());

    nom::branch::alt((
        map(plus, |val| InstrCounter::Relative(val as i64)),
        map(minus, |val| InstrCounter::Relative(-(val as i64))),
        map(absolute, |val| InstrCounter::Absolute(val.into())),
        star,
    )).parse(input)
}

fn parse_calls_line(input: &str) -> IResult<&str, InstrCounter> {
    preceded((tag("calls="), parseu64, space1), parse_costline).parse(input)
}

fn parse_costline(input: &str) -> IResult<&str, InstrCounter> {
    let (input, instr) = parse_subposition(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = parse_till_eol(input)?;
    Ok((input, instr))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        println!("{:#?}", parse_cfn("(23) fooo"));
    }
}
