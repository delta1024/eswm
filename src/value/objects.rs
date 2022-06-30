/*! eswm first class objects.

this Module holds eswm's heap allocated objects.
 */
use std::rc::Rc;
use std::cell::RefCell;
/// The heap allocated object
pub type ObjPtr = Rc<RefCell<dyn ObjVal>>;

#[derive(Debug ,Clone, Copy, PartialOrd, PartialEq)]
pub enum ObjId {
    String
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
    fn as_string(&self) -> Option<&ObjString> {
	None
    }

    fn as_rstring(&self) -> Option<&str> {
	None
    }
}

pub fn print_object(object: &Object) {
    unsafe {
	match object.id {
	    ObjId::String => print!("{}", (*object.object).borrow().as_rstring().unwrap()),
	}
    }	
}

#[derive(Debug)]
/// The Vm's internal representation of a string.
pub struct ObjString(pub String);

impl ObjVal for ObjString {
    fn as_string(&self) -> Option<&ObjString> {
	Some(&self)
    }

    fn as_rstring(&self) -> Option<&str> {
	Some(self.0.as_str())
    }
}

