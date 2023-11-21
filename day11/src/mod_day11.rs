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

#[derive(Error, Debug)]
pub enum MyError {
    #[error("parsing error, {0:?}")]
    Parser(nom::error::ErrorKind),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        Self::Parser(err.code)
    }
}

fn parse_all_monkeys<R: std::io::BufRead>(mut reader: R) -> Result<Vec<Monkey>, MyError> {
    let mut v = Vec::new();
    let mut s = String::new();
    loop {
        v.push(parse_monkey(&mut reader)?);
        match reader.read_line(&mut s) {
            Ok(a) => {if a == 0 {break}}
            Err(_) => break,
        }
    }
    Ok(v)
}

fn parse_monkey<R: std::io::BufRead>(reader: &mut R) -> Result<Monkey, MyError> {
    let mut s = String::new();
    let _ = reader.read_line(&mut s)?;
    let id = parse_monkey_id(&s).finish()?.1;
    s.clear();
    let _ = reader.read_line(&mut s)?;
    let items = parse_starting_items(&s).finish()?.1;
    s.clear();
    let _ = reader.read_line(&mut s)?;
    let op = parse_operation(&s).finish()?.1;
    s.clear();
    let _ = reader.read_line(&mut s)?;
    let div = parse_test_divisible_by(&s).finish()?.1;
    s.clear();
    let _ = reader.read_line(&mut s)?;
    let if_true = parse_test_if_true(&s).finish()?.1;
    s.clear();
    let _ = reader.read_line(&mut s)?;
    let if_false = parse_test_if_false(&s).finish()?.1;

    Ok(Monkey {
        id,
        items,
        op,
        div,
        if_true,
        if_false,
    })
}

fn parse_monkey_id(i: &str) -> IResult<&str, u8> {
    delimited(tag("Monkey "), nom::character::complete::u8, tag(":"))(i)
}

fn parse_starting_items(i: &str) -> IResult<&str, Vec<Item>> {
    preceded(nom::character::complete::space0,
    map(
        preceded(
            tag("Starting items: "),
            separated_list0(tag(", "), nom::character::complete::u32),
        ),
        |v| v.into_iter().map(|x| Item(x)).collect(),
    ))(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    alt((parse_operation_add, parse_operation_multiply))(i)
}

fn parse_operation_add(i: &str) -> IResult<&str, Operation> {
    let op = '+';
    preceded(nom::character::complete::space0,
    map(
        preceded(
            tag("Operation: new = "),
            separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
        ),
        |x| Operation::Add(x.0, x.1),
    ))(i)
}

fn parse_operation_multiply(i: &str) -> IResult<&str, Operation> {
    let op = '*';
    preceded(nom::character::complete::space0,
    map(
        preceded(
            tag("Operation: new = "),
            separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
        ),
        |x| Operation::Multiply(x.0, x.1),
    ))(i)
}

fn parse_term(i: &str) -> IResult<&str, Term> {
    let p_old = map(tag("old"), |_| Term::Old);
    let p_const = map(nom::character::complete::u32, |x| Term::Constant(x));

    alt((p_old, p_const))(i)
}

fn parse_test_divisible_by(i: &str) -> IResult<&str, TestDivisibleBy> {
    preceded(nom::character::complete::space0,
    map(
        preceded(tag("Test: divisible by "), nom::character::complete::u32),
        |x| TestDivisibleBy(x),
    ))(i)
}

fn parse_test_if_true(i: &str) -> IResult<&str, TestIfTrue> {
    preceded(nom::character::complete::space0
        , map(
        preceded(
            tag("If true: throw to monkey "),
            nom::character::complete::u8,
        ),
        |x| TestIfTrue(x),
    ))(i)
}

fn parse_test_if_false(i: &str) -> IResult<&str, TestIfFalse> {
    preceded(nom::character::complete::space0
    ,map(
        preceded(
            tag("If false: throw to monkey "),
            nom::character::complete::u8,
        ),
        |x| TestIfFalse(x),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_monkeys_works() {
        #[rustfmt::skip]
        let s = 
"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

        let mut reader = std::io::BufReader::new(s.as_bytes());

        let v = parse_all_monkeys(reader).unwrap();

        dbg!(&v);
    }
}
