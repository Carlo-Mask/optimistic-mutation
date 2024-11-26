use crate::rc::CowRc;
use std::{
	borrow::{Borrow, Cow},
	ffi::OsStr,
	fmt::{Debug, Display, Formatter},
	ops::Deref,
	ptr,
	rc::Rc,
};
use sugaru::pipeline;

/// Comme un [str] mais [`ToOwned`] donne un [`Rc<str>`] et non un [`String`]
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
		let ptr = pipeline!(string_slice |> ptr::from_mut |> Self::from_str_mut_ptr);
		unsafe { &mut *ptr }
	}

	const fn from_str_ptr(string_slice: *const str) -> *const Self {
		string_slice as _
	}

	const fn from_str_mut_ptr(string_slice: *mut str) -> *mut Self {
		string_slice as _
	}
}

impl ToOwned for ToCowRcStr {
	type Owned = CowRc<str>;

	fn to_owned(&self) -> Self::Owned {
		CowRc::from(&self.str)
	}
}

impl Borrow<ToCowRcStr> for CowRc<str> {
	fn borrow(&self) -> &ToCowRcStr {
		ToCowRcStr::from_str(self)
	}
}

impl Borrow<str> for CowRc<str> {
	fn borrow(&self) -> &str {
		self
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

impl<AsRefStr> AsRef<AsRefStr> for ToCowRcStr
where
	str: AsRef<AsRefStr>,
{
	fn as_ref(&self) -> &AsRefStr {
		self.str.as_ref()
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

impl From<CowRc<str>> for String {
	fn from(value: CowRc<str>) -> Self {
		value.deref().into()
	}
}

impl From<&ToCowRcStr> for Box<str> {
	fn from(value: &ToCowRcStr) -> Self {
		Self::from(&value.str)
	}
}

impl<IntoBoxedStr> From<IntoBoxedStr> for Box<ToCowRcStr>
where
	Box<str>: From<IntoBoxedStr>,
{
	fn from(value: IntoBoxedStr) -> Self {
		unsafe {
			pipeline!(value
				|> Box::from
				|> Box::into_raw
				|> ToCowRcStr::from_str_mut_ptr
				|> Self::from_raw
			)
		}
	}
}

impl From<&ToCowRcStr> for Rc<ToCowRcStr> {
	fn from(value: &ToCowRcStr) -> Self {
		#[allow(unused_braces)]
		unsafe {
			pipeline!({ &value.str }
				|> Rc::<str>::from
				|> Rc::into_raw
				|> ToCowRcStr::from_str_ptr
				|> Self::from_raw
			)
		}
	}
}

impl<'a> From<&'a ToCowRcStr> for Cow<'a, ToCowRcStr> {
	fn from(value: &'a ToCowRcStr) -> Self {
		Cow::Borrowed(value)
	}
}

impl From<CowRc<str>> for Cow<'_, ToCowRcStr> {
	fn from(value: CowRc<str>) -> Self {
		Cow::Owned(value)
	}
}

impl CowRc<str> {
	#[must_use]
	/// Borrows this String as a [`Cow`],
	/// avoiding cloning when the string is not mutated.
	/// Please note cloning is cheap if this Rc is unique.
	/// Use [`DerefMut`] if you're sure you need to mutate
	pub fn borrow_cow(&self) -> Cow<'_, ToCowRcStr> {
		Cow::Borrowed(ToCowRcStr::from_str(self))
	}
}

impl<'a> TryFrom<&'a OsStr> for &'a ToCowRcStr {
	type Error = <&'a str as TryFrom<&'a OsStr>>::Error;

	fn try_from(value: &'a OsStr) -> Result<Self, Self::Error> {
		<&'a str>::try_from(value).map(ToCowRcStr::from_str)
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

impl Default for Box<ToCowRcStr> {
	fn default() -> Self {
		Self::from(<&ToCowRcStr>::default())
	}
}

impl<Item> FromIterator<Item> for CowRc<str>
where
	String: FromIterator<Item>,
{
	fn from_iter<Iterator: IntoIterator<Item = Item>>(iter: Iterator) -> Self {
		pipeline!(iter |> String::from_iter |> Self::from)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn deref_test() {
		let to_rc_str: &ToCowRcStr = ToCowRcStr::from_str("Toto");
		let rc_str: CowRc<str> = to_rc_str.to_owned();
		assert_eq!(rc_str.len(), 4);
	}

	#[test]
	fn cow() {
		let cow: Cow<'_, ToCowRcStr> = pipeline!("toto" |> CowRc::from |> Cow::Owned);
		// Cow deref sur ToCowRcStr qui deref sur str
		assert_eq!(cow.len(), 4); // Le double deref a bien marché
	}
}
