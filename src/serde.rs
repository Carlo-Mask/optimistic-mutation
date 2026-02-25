use crate::rc::CowRc;
use serde::{Serialize, Serializer};
use sugaru::pipeline;

impl<T: ?Sized + Serialize> Serialize for CowRc<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		pipeline!(self |> Self::as_rc).serialize(serializer)
	}
}
