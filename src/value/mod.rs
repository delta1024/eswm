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

pub mod objects; 
use objects::*;
use std::cell::Ref;
use std::cmp::PartialEq;
use std::ops::{Add, Div, Mul, Sub};
#[derive(PartialEq)]
pub enum ValueType {
    Bool,
    Nil,
    Obj,
    Number,
}

#[derive(Debug, Clone, Copy, PartialOrd)]
/// eswm's internal value representation.
pub enum Value {
    Bool(bool),
    Number(f64),
    Obj(Object),
    None,
}

impl Value {
    pub fn is_type(&self, val_type: ValueType) -> bool {
	match *self {
	    Self::Bool(_) => ValueType::Bool == val_type,
	    Self::Number(_) => ValueType::Number == val_type,
	    Self::Obj(_) => ValueType::Obj == val_type,
	    Self::None => ValueType::Nil == val_type,
	}
    }

    pub fn val_type(&self) -> ValueType {
	match self {
	    Self::Bool(_) => ValueType::Bool,
	    Self::Number(_) => ValueType::Number,
	    Self::Obj(_) => ValueType::Obj,
	    Self::None => ValueType::Nil,
	}
    }
    
    pub fn is_obj_type(&self, obj_type: ObjId) -> bool {
	if let Self::Obj(obj) = self {
	    obj.id == obj_type
	} else {
	    false
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

    pub fn as_obj(&self) -> &Object {
	match self {
	    Self::Obj(ref object) => object,
	    _ => unreachable!(),
	}
    }

    pub fn obj_val(&self) -> Ref<'_, (dyn objects::ObjVal + 'static)> {
	unsafe {
	    match self {
		Self::Obj(ref object) => (*object.object).borrow(),
		_ => unreachable!(),
	    }
	}
    }

    pub fn obj_type(&self) -> ObjId {
	match self {
	    Self::Obj(ref object) => object.id,
	    _ => unreachable!(),
	}
    }

    pub fn is_string(&self) -> bool {
	self.is_obj_type(ObjId::String)
    }

    pub fn as_string(&self) -> ObjString {
	ObjString(String::from(self.obj_val().as_rstring().unwrap()))
    }

    pub fn as_rstring(&self) -> String {
	String::from(self.obj_val().as_rstring().unwrap())
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

impl From<Object> for Value {
    fn from(value: Object) -> Value {
	Value::Obj(value)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
	self.val_type() == other.val_type() && 
	    match self.val_type() {
		ValueType::Obj => {
		    let self_type = self.obj_type();
		    let other_type = other.obj_type();	
		    match self.obj_type() {
			ObjId::String if (self_type == other_type) => {
			    self.as_rstring() == other.as_rstring()
			}
			_ => false
		    }
		}
		ValueType::Nil if ValueType::Nil == other.val_type() => true,
		ValueType::Bool if self.as_bool() == other.as_bool() => true,
		ValueType::Number if self.as_number() == other.as_number() => true,
		_ => false
		
	    }

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
	Value::Obj(_) => print_object(value.as_obj()),
    }
}
