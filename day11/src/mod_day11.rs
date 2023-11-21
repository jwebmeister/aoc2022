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
struct Item(u32);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Monkey {
    id: u8,
    items: Vec<Item>,
    op: Operation,
    div: TestDivisibleBy,
    if_true: TestIfTrue,
    if_false: TestIfFalse,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct TestDivisibleBy(u32);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct TestIfTrue(u8);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct TestIfFalse(u8);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Operation {
    Add(Term, Term),
    Multiply(Term, Term),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
            separated_list0(tag(", "), nom::character::complete::u32),
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

fn parse_test_divisible_by(i: &str) -> IResult<&str, TestDivisibleBy> {
    map(
        preceded(
            tag("Test: divisible by "),
            nom::character::complete::u32,
        ),
        |x| TestDivisibleBy(x),
    )(i)
}

fn parse_test_if_true(i: &str) -> IResult<&str, TestIfTrue> {
    map(
        preceded(
            tag("If true: throw to monkey "),
            nom::character::complete::u8,
        ),
        |x| TestIfTrue(x),
    )(i)
}

fn parse_test_if_false(i: &str) -> IResult<&str, TestIfFalse> {
    map(
        preceded(
            tag("If false: throw to monkey "),
            nom::character::complete::u8,
        ),
        |x| TestIfFalse(x),
    )(i)
}