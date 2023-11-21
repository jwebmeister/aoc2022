use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
    Finish, IResult,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Item(i32);

#[derive(Debug, Clone)]
enum Operation {
    Add(Term, Term),
    Multiply(Term, Term),
}

#[derive(Clone, Copy, Debug)]
pub enum Term {
    Old,
    Constant(u32),
}

fn parse_monkey_id(i: &str) -> IResult<&str, u8> {
    delimited(tag("Monkey "), nom::character::complete::u8, tag(":"))(i)
}

fn parse_starting_items(i: &str) -> IResult<&str, Vec<Item>> {
    map(
        preceded(
            tag("Starting items: "),
            separated_list0(tag(", "), nom::character::complete::i32),
        ),
        |v| v.into_iter().map(|x| Item(x)).collect(),
    )(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    alt((parse_operation_add, parse_operation_multiply))(i)
}

fn parse_operation_add(i: &str) -> IResult<&str, Operation> {
    let op = '+';
    map(
        preceded(
            tag("Operation: new = "),
            separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
        ),
        |x| Operation::Add(x.0, x.1),
    )(i)
}

fn parse_operation_multiply(i: &str) -> IResult<&str, Operation> {
    let op = '*';
    map(
        preceded(
            tag("Operation: new = "),
            separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
        ),
        |x| Operation::Multiply(x.0, x.1),
    )(i)
}

fn parse_term(i: &str) -> IResult<&str, Term> {
    let p_old = map(tag("old"), |_| Term::Old);
    let p_const = map(nom::character::complete::u32, |x| Term::Constant(x));

    alt((p_old, p_const))(i)
}
