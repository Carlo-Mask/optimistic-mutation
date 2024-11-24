use crate::rc::CowRc;
use std::{
	borrow::Borrow,
	fmt::{Debug, Display, Formatter},
	ops::Deref,
	ptr,
	rc::Rc,
};
use sugaru::pipeline;

/// Comme un [str] mais [ToOwned] donne un [Rc<str>] et non un [String]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct ToCowRcStr {
	pub str: str,
}

impl ToCowRcStr {
	pub const fn from_str(string_slice: &str) -> &Self {
		let ptr = ptr::from_ref(string_slice) as *const Self;
		unsafe { &*ptr }
	}

	pub fn from_str_mut(string_slice: &mut str) -> &mut Self {
		let ptr = ptr::from_mut(string_slice) as *mut Self;
		unsafe { &mut *ptr }
	}
}

impl ToOwned for ToCowRcStr {
	type Owned = Rc<str>;

	fn to_owned(&self) -> Self::Owned {
		Rc::from(&self.str)
	}
}

impl Borrow<ToCowRcStr> for Rc<str> {
	fn borrow(&self) -> &ToCowRcStr {
		ToCowRcStr::from_str(self)
	}
}

impl Deref for ToCowRcStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.str
	}
}

impl AsRef<str> for ToCowRcStr {
	fn as_ref(&self) -> &str {
		self
	}
}

impl<'a> From<&'a str> for &'a ToCowRcStr {
	fn from(value: &'a str) -> Self {
		ToCowRcStr::from_str(value)
	}
}

impl<'a> From<&'a ToCowRcStr> for &'a str {
	fn from(value: &'a ToCowRcStr) -> Self {
		&value.str
	}
}

impl<ComparableToStr: ?Sized> PartialEq<ComparableToStr> for ToCowRcStr
where
	str: PartialEq<ComparableToStr>,
{
	fn eq(&self, other: &ComparableToStr) -> bool {
		self.str == *other
	}
}

impl Display for ToCowRcStr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.str, f)
	}
}

impl Default for &ToCowRcStr {
	fn default() -> Self {
		ToCowRcStr::from_str(<&str>::default())
	}
}

impl<'a, Item> FromIterator<Item> for &'a ToCowRcStr
where
	&'a str: FromIterator<Item>,
{
	fn from_iter<Iterator: IntoIterator<Item = Item>>(iter: Iterator) -> Self {
		pipeline!(iter => <&str>::from_iter => ToCowRcStr::from_str)
	}
}

impl<Item> FromIterator<Item> for CowRc<str>
where
	String: FromIterator<Item>,
{
	fn from_iter<Iterator: IntoIterator<Item = Item>>(iter: Iterator) -> Self {
		pipeline!(iter => String::from_iter => Self::from)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn deref_test() {
		let to_rc_str: &ToCowRcStr = ToCowRcStr::from_str("Toto");
		let rc_str: Rc<str> = to_rc_str.to_owned();
		assert_eq!(rc_str.len(), 4);
	}
}
