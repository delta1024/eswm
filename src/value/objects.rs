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

/*! eswm first class objects.

this Module holds eswm's heap allocated objects.
 */
use std::rc::Rc;
use std::cell::RefCell;
/// The heap allocated object
pub type ObjPtr = Rc<RefCell<dyn ObjVal>>;

#[derive(Debug ,Clone, Copy, PartialOrd, PartialEq)]
pub enum ObjId {

}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Object {
    pub id: ObjId,
    /// A pointer to the [`Rc`] of the object
    pub object: *const ObjPtr,
}

impl Object {
    pub fn new(id: ObjId, object: *const ObjPtr ) -> Object {
	Object {
	    id,
	    object,
	}
    }
}

/// A link list that holds the master [`Rc`] for each eswm object.
pub struct ObjList {
    pub value: ObjPtr,
    pub next: Option<Box<ObjList>>,
}


/// Defines Object behavior.
pub trait ObjVal {
    fn placeholder_fn();
}

pub fn print_object(object: &Object) {
    // unsafe {
    // 	match object.id {

    // 	}
    // }	
}



