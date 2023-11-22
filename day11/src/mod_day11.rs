use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, one_of, space0, space1},
    combinator::{map, map_parser},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};
use std::collections::VecDeque;
use thiserror::Error;

const MONKEY_GETS_BORED: bool = false;

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
    pub round: usize,
    pub data: Vec<Monkey>,
    pub div_product: u64,
}

impl MonkeyList {
    pub fn process_monkey(&mut self, id: usize) -> Result<(), MyError> {
        let monkey = &mut self.data[id];
        let to_send_items = monkey.complete_turn(self.div_product)?;

        for send_item in to_send_items {
            self.data[send_item.0 as usize].items.push_back(send_item.1);
        }
        Ok(())
    }

    pub fn complete_round(&mut self) -> Result<usize, MyError> {
        for i in 0..self.data.len() {
            self.process_monkey(i)?;
        }
        self.round += 1;
        Ok(self.round)
    }

    pub fn set_div_product(&mut self) {
        self.div_product = self.data.iter().map(|m| m.div.0).product();
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
    pub num_items_inspected: u64,
}

impl Monkey {
    pub fn process_one_item(&mut self, div_product: u64) -> Result<(u8, Item), MyError> {
        let mut item = self.items.pop_front().ok_or(MyError::EmptyItems)?;
        item.inspect(&self.op);
        if MONKEY_GETS_BORED {
            item.bored_with();
        };
        item.0 %= div_product;
        let send_to = item.where_to_throw(&self.div, &self.if_true, &self.if_false);
        Ok((send_to, item))
    }

    pub fn complete_turn(&mut self, div_product: u64) -> Result<Vec<(u8, Item)>, MyError> {
        let mut v = Vec::new();
        while !self.items.is_empty() {
            v.push(self.process_one_item(div_product)?);
            self.num_items_inspected += 1;
        }
        Ok(v)
    }
}

impl std::fmt::Debug for Monkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Monkey {0}: Inspected={1}, {2:?}",
            &self.id, &self.num_items_inspected, &self.items
        )?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Item(u64);

impl Item {
    pub fn inspect(&mut self, op: &Operation) {
        self.0 = op.eval(self.0);
    }

    pub fn bored_with(&mut self) {
        self.0 /= 3;
    }

    fn _test_divisible_by(&self, div: &TestDivisibleBy) -> bool {
        self.0 % div.0 == 0
    }

    pub fn where_to_throw(
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
pub struct TestDivisibleBy(u64);

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
    pub fn eval(&self, old: u64) -> u64 {
        match self {
            Operation::Add(a, b) => a.eval(old) + b.eval(old),
            Operation::Multiply(a, b) => a.eval(old) * b.eval(old),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Term {
    Old,
    Constant(u64),
}

impl Term {
    pub fn eval(&self, old: u64) -> u64 {
        match &self {
            Term::Old => old,
            Term::Constant(x) => *x,
        }
    }
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
    let mut ml = MonkeyList {
        round: 0,
        data: v,
        div_product: 0,
    };
    ml.set_div_product();
    Ok(ml)
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
        num_items_inspected: 0,
    })
}

fn parse_monkey_id(i: &str) -> IResult<&str, u8> {
    delimited(tag("Monkey "), nom::character::complete::u8, tag(":"))(i)
}

fn parse_starting_items(i: &str) -> IResult<&str, VecDeque<Item>> {
    preceded(
        space0,
        map(
            preceded(
                tag("Starting items: "),
                separated_list0(tag(", "), nom::character::complete::u64),
            ),
            |v| v.into_iter().map(Item).collect::<VecDeque<Item>>(),
        ),
    )(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    preceded(
        space0,
        map(
            preceded(
                tag("Operation: new = "),
                tuple((
                    parse_term,
                    preceded(space1, one_of("+*")),
                    preceded(space1, parse_term),
                )),
            ),
            |x| match x.1 {
                '+' => Operation::Add(x.0, x.2),
                '*' => Operation::Multiply(x.0, x.2),
                _ => {
                    dbg!("Op char is not + or *");
                    unreachable!()
                }
            },
        ),
    )(i)
}

fn parse_term(i: &str) -> IResult<&str, Term> {
    let p_old = map(tag("old"), |_| Term::Old);
    let p_digit = map_parser(digit1, nom::character::complete::u64);
    let p_const = map(p_digit, Term::Constant);

    alt((p_old, p_const))(i)
}

fn parse_test_divisible_by(i: &str) -> IResult<&str, TestDivisibleBy> {
    preceded(
        space0,
        map(
            preceded(tag("Test: divisible by "), nom::character::complete::u64),
            TestDivisibleBy,
        ),
    )(i)
}

fn parse_test_if_true(i: &str) -> IResult<&str, TestIfTrue> {
    preceded(
        space0,
        map(
            preceded(
                tag("If true: throw to monkey "),
                nom::character::complete::u8,
            ),
            TestIfTrue,
        ),
    )(i)
}

fn parse_test_if_false(i: &str) -> IResult<&str, TestIfFalse> {
    preceded(
        space0,
        map(
            preceded(
                tag("If false: throw to monkey "),
                nom::character::complete::u8,
            ),
            TestIfFalse,
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_operation_works() {
        let s = "  Operation: new = old * 19\n";
        let r = Operation::Multiply(Term::Old, Term::Constant(19));

        let op = parse_operation(s).unwrap().1;

        assert_eq!(op, r);
    }

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
                num_items_inspected: 0,
            },
            Monkey {
                id: 1,
                items: VecDeque::from([Item(54), Item(65), Item(75), Item(74)]),
                op: Operation::Add(Term::Old, Term::Constant(6)),
                div: TestDivisibleBy(19),
                if_true: TestIfTrue(2),
                if_false: TestIfFalse(0),
                num_items_inspected: 0,
            },
            Monkey {
                id: 2,
                items: VecDeque::from([Item(79), Item(60), Item(97)]),
                op: Operation::Multiply(Term::Old, Term::Old),
                div: TestDivisibleBy(13),
                if_true: TestIfTrue(1),
                if_false: TestIfFalse(3),
                num_items_inspected: 0,
            },
            Monkey {
                id: 3,
                items: VecDeque::from([Item(74)]),
                op: Operation::Add(Term::Old, Term::Constant(3)),
                div: TestDivisibleBy(17),
                if_true: TestIfTrue(0),
                if_false: TestIfFalse(1),
                num_items_inspected: 0,
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
                    num_items_inspected: 0,
                },
                Monkey {
                    id: 1,
                    items: VecDeque::from([Item(54), Item(65), Item(75), Item(74)]),
                    op: Operation::Add(Term::Old, Term::Constant(6)),
                    div: TestDivisibleBy(19),
                    if_true: TestIfTrue(2),
                    if_false: TestIfFalse(0),
                    num_items_inspected: 0,
                },
                Monkey {
                    id: 2,
                    items: VecDeque::from([Item(79), Item(60), Item(97)]),
                    op: Operation::Multiply(Term::Old, Term::Old),
                    div: TestDivisibleBy(13),
                    if_true: TestIfTrue(1),
                    if_false: TestIfFalse(3),
                    num_items_inspected: 0,
                },
                Monkey {
                    id: 3,
                    items: VecDeque::from([Item(74)]),
                    op: Operation::Add(Term::Old, Term::Constant(3)),
                    div: TestDivisibleBy(17),
                    if_true: TestIfTrue(0),
                    if_false: TestIfFalse(1),
                    num_items_inspected: 0,
                },
            ],
            div_product: 0,
        };

        ml.set_div_product();

        if MONKEY_GETS_BORED {
            while ml.round < 20 {
                ml.complete_round().unwrap();
            }

            let mut iter = ml.data.iter().map(|m| m.num_items_inspected);

            assert_eq!(101, iter.next().unwrap());
            assert_eq!(95, iter.next().unwrap());
            assert_eq!(7, iter.next().unwrap());
            assert_eq!(105, iter.next().unwrap());

            let r0_items = ml.data[0].items.iter().map(|x| x.0).collect::<Vec<_>>();
            let r1_items = ml.data[1].items.iter().map(|x| x.0).collect::<Vec<_>>();
            let r2_items = ml.data[2].items.iter().map(|x| x.0).collect::<Vec<_>>();
            let r3_items = ml.data[3].items.iter().map(|x| x.0).collect::<Vec<_>>();

            assert_eq!(vec![10, 12, 14, 26, 34], r0_items);
            assert_eq!(vec![245, 93, 53, 199, 115], r1_items);
            assert_eq!(Vec::<u64>::new(), r2_items);
            assert_eq!(Vec::<u64>::new(), r3_items);

            let mut v_num_inspected = ml
                .data
                .into_iter()
                .map(|m| m.num_items_inspected)
                .collect::<Vec<_>>();
            v_num_inspected.sort_by(|a, b| b.cmp(a));
            let monkey_business = v_num_inspected[0..=1]
                .iter()
                .copied()
                .reduce(|acc, el| acc * el)
                .unwrap();
            assert_eq!(10605, monkey_business);
        } else {
            while ml.round < 10_000 {
                ml.complete_round().unwrap();
            }
            let mut v_num_inspected = ml
                .data
                .into_iter()
                .map(|m| m.num_items_inspected)
                .collect::<Vec<_>>();
            v_num_inspected.sort_by(|a, b| b.cmp(a));
            let monkey_business = v_num_inspected[0..=1]
                .iter()
                .copied()
                .reduce(|acc, el| acc * el)
                .unwrap();
            assert_eq!(2713310158, monkey_business);
        }
    }
}
