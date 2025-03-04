use crate::rc::CowRc;
use serde::{Serialize, Serializer};
use std::ops::Deref;

impl<T: ?Sized + Serialize> Serialize for CowRc<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.deref().serialize(serializer)
	}
}
