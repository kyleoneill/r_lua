use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::rc::Rc;

use crate::ast::lua_program::{BooleanOperator, MathOperator};
use crate::err_handle::RuntimeFailure;
use crate::frontend::Context;

pub struct Data {
    // Data is wrapped in a Rc<RefCell<>>
    // The Rc is a reference counter which allows for multiple references to the inner value
    // This is necessary to prevent variables from being cloned every time they are read, .clone()
    // on an rc will just increment the reference count.
    // The RefCell allows for data mutation in multiple places
    handle: Rc<RefCell<DataKind>>
}

impl Data {
    pub fn new(data_kind: DataKind) -> Self {
        Self {
            handle: Rc::new(RefCell::new(data_kind)),
        }
    }
    pub fn borrow(&self, context: &Context) -> Result<Ref<DataKind>, RuntimeFailure> {
        match self.handle.try_borrow() {
            // Must return a Ref<T> here, returning a Ref<T>::deref() will error.
            // This happens because RefCell<T>::try_borrow returns a Ref<T> with the lifetime of the &self passed into
            // this method. Calling deref on that Ref<T> will be a borrow of a borrow, where the second borrow will
            // go out of scope when this function ends. The first borrow has the lifetime of &self and can be
            // returned, because the caller gave us &self and knows what the lifetime is.
            Ok(d) => Ok(d),
            Err(_) => Err(RuntimeFailure::BorrowError(
                "Cannot borrow a variable when it has a mutable reference in use".to_owned(),
                context.current_line
            )),
        }
    }
    pub fn borrow_mut(&self, context: &Context) -> Result<RefMut<DataKind>, RuntimeFailure> {
        match self.handle.try_borrow_mut() {
            Ok(d) => Ok(d),
            Err(_) => Err(RuntimeFailure::BorrowError(
                "Cannot borrow a variable mutably when it already has a reference in use"
                    .to_owned(),
                context.current_line
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataKind {
    String(String),
    Number(NumberKind),
    Bool(bool),
    Null,
    // TODO: Table(HashMap<String, DataKind>)
}

impl DataKind {
    pub fn math_binary_op(&self, other: &Self, op: MathOperator, context: &Context) -> Result<NumberKind, RuntimeFailure> {
        let (l_num, r_num) = Self::both_numerical(context, self, other)?;
        let l_copy = l_num.clone();
        let r_copy = r_num.clone();
        match op {
            MathOperator::Plus => return Ok(l_copy.add(r_copy)),
            MathOperator::Minus => return Ok(l_copy.sub(r_copy)),
            MathOperator::Multiply => todo!("mul"),
            MathOperator::FloatDivision => todo!("float_div"),
            MathOperator::FloorDivision => todo!("floor_div"),
            MathOperator::Exponent => todo!("exp"),
            MathOperator::Mod => todo!("mod"),
        }
    }
    pub fn boolean_binary_op(&self, other: &Self, op: BooleanOperator, context: &Context) -> Result<bool, RuntimeFailure> {
        match op {
            BooleanOperator::LessThan => {
                let (l_num, r_num) = Self::both_numerical(context, self, other)?;
                Ok(l_num < r_num)
            },
            BooleanOperator::LessThanEqualTo => {
                let (l_num, r_num) = Self::both_numerical(context, self, other)?;
                Ok(l_num <= r_num)
            },
            BooleanOperator::GreaterThan => {
                let (l_num, r_num) = Self::both_numerical(context, self, other)?;
                Ok(l_num > r_num)
            },
            BooleanOperator::GreaterThanEqualTo => {
                let (l_num, r_num) = Self::both_numerical(context, self, other)?;
                Ok(l_num >= r_num)
            },
            BooleanOperator::Equal => Ok(self == other),
            BooleanOperator::Unequal => Ok(self != other),
            BooleanOperator::And => Ok(self.is_true() && other.is_true()),
            BooleanOperator::Or => Ok(self.is_true() || other.is_true()),
        }
    }
    fn both_numerical<'a>(context: &Context, first: &'a Self, second: &'a Self) -> Result<(&'a NumberKind, &'a NumberKind), RuntimeFailure> {
        if let DataKind::Number(l_num) = first {
            if let DataKind::Number(r_num) = second {
                return Ok((l_num, r_num))
            }
        }
        Err(RuntimeFailure::WrongType("Number".to_string(), context.current_line))
    }
    pub fn is_true(&self) -> bool {
        match self {
            DataKind::String(string) => !string.is_empty(),
            DataKind::Number(num) => {
                match num {
                    NumberKind::Integer(int) => *int != 0,
                    NumberKind::Float(float) => *float != 0.0f64
                }
            },
            DataKind::Bool(bool) => *bool,
            DataKind::Null => false
        }
    }
}

impl Display for DataKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataKind::String(string) => write!(f, "{}", string),
            DataKind::Number(num) => write!(f, "{}", num),
            DataKind::Bool(bool) => write!(f, "{}", bool),
            DataKind::Null => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NumberKind {
    Integer(i64),
    Float(f64)
}

impl Display for NumberKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberKind::Integer(int) => write!(f, "{}", int),
            NumberKind::Float(float) => write!(f, "{}", float),
        }
    }
}

impl Add for NumberKind {
    type Output = NumberKind;

    fn add(self, rhs: Self) -> Self::Output {
        // There has to be a better way to do this
        match self {
            NumberKind::Integer(lhs_int) => {
                match rhs {
                    NumberKind::Integer(rhs_int) => {
                        NumberKind::Integer(lhs_int + rhs_int)
                    },
                    NumberKind::Float(rhs_float) => {
                        NumberKind::Float(lhs_int as f64 + rhs_float)
                    }
                }
            },
            NumberKind::Float(lhs_float) => {
                match rhs {
                    NumberKind::Integer(rhs_int) => {
                        NumberKind::Float(lhs_float + rhs_int as f64)
                    },
                    NumberKind::Float(rhs_float) => {
                        NumberKind::Float(lhs_float + rhs_float)
                    }
                }
            }
        }
    }
}

impl Sub for NumberKind {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // There has to be a better way to do this
        match self {
            NumberKind::Integer(lhs_int) => {
                match rhs {
                    NumberKind::Integer(rhs_int) => {
                        NumberKind::Integer(lhs_int - rhs_int)
                    },
                    NumberKind::Float(rhs_float) => {
                        NumberKind::Float(lhs_int as f64 - rhs_float)
                    }
                }
            },
            NumberKind::Float(lhs_float) => {
                match rhs {
                    NumberKind::Integer(rhs_int) => {
                        NumberKind::Float(lhs_float - rhs_int as f64)
                    },
                    NumberKind::Float(rhs_float) => {
                        NumberKind::Float(lhs_float - rhs_float)
                    }
                }
            }
        }
    }
}
