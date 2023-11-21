use std::collections::VecDeque;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
    Finish, IResult,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("parsing error, {0:?}")]
    Parser(nom::error::ErrorKind),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Tried to process item from monkey with no items!")]
    EmptyItems,
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        Self::Parser(err.code)
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct MonkeyList {
    round: usize,
    data: Vec<Monkey>,
}

impl MonkeyList {
    fn process_monkey(&mut self, id: usize) -> Result<(), MyError> {
        let monkey = &mut self.data[id as usize];
        let to_send_items = monkey.complete_turn()?;

        for send_item in to_send_items {
            self.data[send_item.0 as usize].items.push_back(send_item.1);
        }
        Ok(())
    }

    fn complete_round(&mut self) -> Result<(), MyError> {
        for i in 0..self.data.len() {
            self.process_monkey(i)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for MonkeyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Round: {}", self.round)?;
        for monkey in &self.data {
            writeln!(f, "{0:?}", monkey)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Monkey {
    pub id: u8,
    pub items: VecDeque<Item>,
    pub op: Operation,
    pub div: TestDivisibleBy,
    pub if_true: TestIfTrue,
    pub if_false: TestIfFalse,
}

impl Monkey {
    fn process_one_item(&mut self) -> Result<(u8, Item), MyError> {
        let mut item = self.items.pop_front().ok_or(MyError::EmptyItems)?;
        item.inspect(&self.op);
        item.bored_with();
        let send_to = item.where_to_throw(&self.div, &self.if_true, &self.if_false);
        Ok((send_to, item))
    }

    fn complete_turn(&mut self) -> Result<Vec<(u8, Item)>, MyError> {
        let mut v = Vec::new();
        while !self.items.is_empty() {
            v.push(self.process_one_item()?);
        }
        Ok(v)
    }
}

impl std::fmt::Debug for Monkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Monkey {0}: {1:?}", &self.id, &self.items)?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Item(u32);

impl Item {
    fn inspect(&mut self, op: &Operation) {
        self.0 = op.op_result(self.0);
    }

    fn bored_with(&mut self) {
        self.0 /= 3;
    }

    fn test_divisible_by(&self, div: &TestDivisibleBy) -> bool {
        self.0 % div.0 == 0
    }

    fn where_to_throw(
        &self,
        div: &TestDivisibleBy,
        if_true: &TestIfTrue,
        if_false: &TestIfFalse,
    ) -> u8 {
        match self.0 % div.0 == 0 {
            true => if_true.0,
            false => if_false.0,
        }
    }
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0:?}", &self.0)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TestDivisibleBy(u32);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TestIfTrue(u8);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TestIfFalse(u8);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Operation {
    Add(Term, Term),
    Multiply(Term, Term),
}

impl Operation {
    fn op_result(&self, old: u32) -> u32 {
        match self {
            Operation::Add(a, b) => {
                let a_n = match a {
                    Term::Old => old,
                    Term::Constant(a_n) => *a_n,
                };
                let b_n = match b {
                    Term::Old => old,
                    Term::Constant(b_n) => *b_n,
                };
                a_n + b_n
            }
            Operation::Multiply(a, b) => {
                let a_n = match a {
                    Term::Old => old,
                    Term::Constant(a_n) => *a_n,
                };
                let b_n = match b {
                    Term::Old => old,
                    Term::Constant(b_n) => *b_n,
                };
                a_n * b_n
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Term {
    Old,
    Constant(u32),
}

pub fn parse_all_monkeys<R: std::io::BufRead>(mut reader: R) -> Result<MonkeyList, MyError> {
    let mut v = Vec::new();
    let mut s = String::new();
    loop {
        v.push(parse_monkey(&mut reader)?);
        match reader.read_line(&mut s) {
            Ok(a) => {
                if a == 0 {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(MonkeyList { round: 0, data: v })
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

fn parse_starting_items(i: &str) -> IResult<&str, VecDeque<Item>> {
    preceded(
        nom::character::complete::space0,
        map(
            preceded(
                tag("Starting items: "),
                separated_list0(tag(", "), nom::character::complete::u32),
            ),
            |v| v.into_iter().map(|x| Item(x)).collect::<VecDeque<Item>>(),
        ),
    )(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    alt((parse_operation_add, parse_operation_multiply))(i)
}

fn parse_operation_add(i: &str) -> IResult<&str, Operation> {
    let op = '+';
    preceded(
        nom::character::complete::space0,
        map(
            preceded(
                tag("Operation: new = "),
                separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
            ),
            |x| Operation::Add(x.0, x.1),
        ),
    )(i)
}

fn parse_operation_multiply(i: &str) -> IResult<&str, Operation> {
    let op = '*';
    preceded(
        nom::character::complete::space0,
        map(
            preceded(
                tag("Operation: new = "),
                separated_pair(parse_term, tag(format!(" {} ", op).as_str()), parse_term),
            ),
            |x| Operation::Multiply(x.0, x.1),
        ),
    )(i)
}

fn parse_term(i: &str) -> IResult<&str, Term> {
    let p_old = map(tag("old"), |_| Term::Old);
    let p_const = map(nom::character::complete::u32, |x| Term::Constant(x));

    alt((p_old, p_const))(i)
}

fn parse_test_divisible_by(i: &str) -> IResult<&str, TestDivisibleBy> {
    preceded(
        nom::character::complete::space0,
        map(
            preceded(tag("Test: divisible by "), nom::character::complete::u32),
            |x| TestDivisibleBy(x),
        ),
    )(i)
}

fn parse_test_if_true(i: &str) -> IResult<&str, TestIfTrue> {
    preceded(
        nom::character::complete::space0,
        map(
            preceded(
                tag("If true: throw to monkey "),
                nom::character::complete::u8,
            ),
            |x| TestIfTrue(x),
        ),
    )(i)
}

fn parse_test_if_false(i: &str) -> IResult<&str, TestIfFalse> {
    preceded(
        nom::character::complete::space0,
        map(
            preceded(
                tag("If false: throw to monkey "),
                nom::character::complete::u8,
            ),
            |x| TestIfFalse(x),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::collections::vec_deque;

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

        let r = vec![
            Monkey {
                id: 0,
                items: VecDeque::from([Item(79), Item(98)]),
                op: Operation::Multiply(Term::Old, Term::Constant(19)),
                div: TestDivisibleBy(23),
                if_true: TestIfTrue(2),
                if_false: TestIfFalse(3),
            },
            Monkey {
                id: 1,
                items: VecDeque::from([Item(54), Item(65), Item(75), Item(74)]),
                op: Operation::Add(Term::Old, Term::Constant(6)),
                div: TestDivisibleBy(19),
                if_true: TestIfTrue(2),
                if_false: TestIfFalse(0),
            },
            Monkey {
                id: 2,
                items: VecDeque::from([Item(79), Item(60), Item(97)]),
                op: Operation::Multiply(Term::Old, Term::Old),
                div: TestDivisibleBy(13),
                if_true: TestIfTrue(1),
                if_false: TestIfFalse(3),
            },
            Monkey {
                id: 3,
                items: VecDeque::from([Item(74)]),
                op: Operation::Add(Term::Old, Term::Constant(3)),
                div: TestDivisibleBy(17),
                if_true: TestIfTrue(0),
                if_false: TestIfFalse(1),
            },
        ];

        let reader = std::io::BufReader::new(s.as_bytes());

        let v = parse_all_monkeys(reader).unwrap();

        assert_eq!(r, v.data);
    }

    #[test]
    fn monkey_list_complete_round_works() {
        let mut ml = MonkeyList {
            round: 0,
            data: vec![
                Monkey {
                    id: 0,
                    items: VecDeque::from([Item(79), Item(98)]),
                    op: Operation::Multiply(Term::Old, Term::Constant(19)),
                    div: TestDivisibleBy(23),
                    if_true: TestIfTrue(2),
                    if_false: TestIfFalse(3),
                },
                Monkey {
                    id: 1,
                    items: VecDeque::from([Item(54), Item(65), Item(75), Item(74)]),
                    op: Operation::Add(Term::Old, Term::Constant(6)),
                    div: TestDivisibleBy(19),
                    if_true: TestIfTrue(2),
                    if_false: TestIfFalse(0),
                },
                Monkey {
                    id: 2,
                    items: VecDeque::from([Item(79), Item(60), Item(97)]),
                    op: Operation::Multiply(Term::Old, Term::Old),
                    div: TestDivisibleBy(13),
                    if_true: TestIfTrue(1),
                    if_false: TestIfFalse(3),
                },
                Monkey {
                    id: 3,
                    items: VecDeque::from([Item(74)]),
                    op: Operation::Add(Term::Old, Term::Constant(3)),
                    div: TestDivisibleBy(17),
                    if_true: TestIfTrue(0),
                    if_false: TestIfFalse(1),
                },
            ],
        };

        ml.complete_round().unwrap();

        dbg!(&ml);
    }
}
