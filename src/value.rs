// eswm -- Emacs Standalown WindowManager
// Copyright (C) 2022 Jacob Stannix

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::ops::{Add, Div, Mul, Sub};
#[derive(PartialEq)]
pub enum ValueType {
    Bool,
    Nil,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// eswm's internal value representation.
pub enum Value {
    Bool(bool),
    Number(f64),
    None,
}

impl Value {
    pub fn is_type(&self, val_type: ValueType) -> bool {
	match *self {
	    Self::Bool(_) => ValueType::Bool == val_type,
	    Self::Number(_) => ValueType::Number == val_type,
	    Self::None => ValueType::Nil == val_type,
	}
    }

    pub fn nil() -> Value {
	Value::None
    }

    pub fn as_bool(&self) -> bool {
	match *self {
	    Self::Bool(val) => val,
	    _ => unreachable!(),
	}
    }

    pub fn as_number(&self) -> f64 {
	match *self {
	    Self::Number(val) => val,
	    _ => unreachable!(),
	}
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Value {
	Value::Number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
	Value::Bool(value)
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
	match self {
	    Self::Number(val) => match other {
		Self::Number(val2) => (val + val2).into(),
		_ => unreachable!(),
	    },
	    _ => unreachable!(),
	}
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
	match self {
	    Self::Number(val) => match other {
		Self::Number(val2) => (val - val2).into(),
		_ => unreachable!(),
	    },
	    _ => unreachable!(),
	}
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
	match self {
	    Self::Number(val) => match other {
		Self::Number(val2) => (val / val2).into(),
		_ => unreachable!(),
	    },
	    _ => unreachable!(),
	}
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
	match self {
	    Self::Number(val) => match other {
		Self::Number(val2) => (val * val2).into(),
		_ => unreachable!(),
	    },
	    _ => unreachable!(),
	}
    }
}

/// Prints [`Value`] to stdout.
pub fn print_value(value: Value) {
    match value {
	Value::None => print!("nil"),
	Value::Bool(_) => print!("{}", value.as_bool()),
	Value::Number(_) => print!("{}", value.as_number()),
    }
}
