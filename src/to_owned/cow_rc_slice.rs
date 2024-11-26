use crate::rc::CowRc;
use std::{
	borrow::{Borrow, Cow},
	fmt::Debug,
	ops::Deref,
	ptr,
	rc::Rc,
};
use sugaru::pipeline;

/// Comme un [[T]] mais [`ToOwned`] donne un [`Rc<[T]>`] et non un [`String`]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct ToCowRcSlice<T> {
	pub slice: [T],
}

impl<T> ToCowRcSlice<T> {
	pub fn from_array<const N: usize>(array: &[T; N]) -> &Self {
		Self::from_slice(&array[..])
	}

	pub const fn from_slice(slice: &[T]) -> &Self {
		let ptr = ptr::from_ref(slice) as *const Self;
		unsafe { &*ptr }
	}

	pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
		let ptr = pipeline!(slice |> ptr::from_mut |> Self::from_slice_mut_ptr);
		unsafe { &mut *ptr }
	}

	const fn from_slice_mut_ptr(slice: *mut [T]) -> *mut Self {
		slice as _
	}

	const fn from_slice_ptr(slice: *const [T]) -> *const Self {
		slice as _
	}
}

impl<T: Clone> ToOwned for ToCowRcSlice<T> {
	type Owned = CowRc<[T]>;

	fn to_owned(&self) -> Self::Owned {
		CowRc::from(&self.slice)
	}
}

impl<T> Borrow<ToCowRcSlice<T>> for CowRc<[T]> {
	fn borrow(&self) -> &ToCowRcSlice<T> {
		ToCowRcSlice::from_slice(self)
	}
}

impl<T> Borrow<[T]> for CowRc<[T]> {
	fn borrow(&self) -> &[T] {
		self
	}
}

impl<T> Deref for ToCowRcSlice<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&self.slice
	}
}

impl<T, AsRefSlice: ?Sized> AsRef<AsRefSlice> for ToCowRcSlice<T>
where
	[T]: AsRef<AsRefSlice>,
{
	fn as_ref(&self) -> &AsRefSlice {
		self.slice.as_ref()
	}
}

impl<'a, T> From<&'a [T]> for &'a ToCowRcSlice<T> {
	fn from(value: &'a [T]) -> Self {
		ToCowRcSlice::from_slice(value)
	}
}

impl<'a, T> From<&'a ToCowRcSlice<T>> for &'a [T] {
	fn from(value: &'a ToCowRcSlice<T>) -> Self {
		&value.slice
	}
}

impl<T: Clone> From<CowRc<[T]>> for Vec<T> {
	fn from(value: CowRc<[T]>) -> Self {
		value.deref().into()
	}
}

impl<T: Clone> From<&ToCowRcSlice<T>> for Box<[T]> {
	fn from(value: &ToCowRcSlice<T>) -> Self {
		Self::from(&value.slice)
	}
}

impl<T, IntoBoxedSlice> From<IntoBoxedSlice> for Box<ToCowRcSlice<T>>
where
	Box<[T]>: From<IntoBoxedSlice>,
{
	fn from(value: IntoBoxedSlice) -> Self {
		unsafe {
			pipeline!(value
				|> Box::from
				|> Box::into_raw
				|> ToCowRcSlice::from_slice_mut_ptr
				|> Self::from_raw
			)
		}
	}
}

impl<T: Clone> From<&ToCowRcSlice<T>> for Rc<ToCowRcSlice<T>> {
	fn from(value: &ToCowRcSlice<T>) -> Self {
		#[allow(unused_braces)]
		unsafe {
			pipeline!({ &value.slice }
				|> Rc::<[T]>::from
				|> Rc::into_raw
				|> ToCowRcSlice::from_slice_ptr
				|> Self::from_raw
			)
		}
	}
}

impl<'a, T: Clone> From<&'a ToCowRcSlice<T>> for Cow<'a, ToCowRcSlice<T>> {
	fn from(value: &'a ToCowRcSlice<T>) -> Self {
		Cow::Borrowed(value)
	}
}

impl<T: Clone> From<CowRc<[T]>> for Cow<'_, ToCowRcSlice<T>> {
	fn from(value: CowRc<[T]>) -> Self {
		Cow::Owned(value)
	}
}

impl<T: Clone> CowRc<[T]> {
	#[must_use]
	/// Borrows this slice as a [`Cow`],
	/// avoiding cloning when the slice is not mutated.
	/// Please note cloning is cheap if this Rc is unique.
	/// Use [`DerefMut`] if you're sure you need to mutate
	pub fn borrow_cow(&self) -> Cow<'_, ToCowRcSlice<T>> {
		Cow::Borrowed(ToCowRcSlice::from_slice(self))
	}
}

impl<T, ComparableToSlice: ?Sized> PartialEq<ComparableToSlice> for ToCowRcSlice<T>
where
	[T]: PartialEq<ComparableToSlice>,
{
	fn eq(&self, other: &ComparableToSlice) -> bool {
		self.slice == *other
	}
}

impl<T> Default for &ToCowRcSlice<T> {
	fn default() -> Self {
		ToCowRcSlice::from_slice(<&[T]>::default())
	}
}

impl<T> Default for Box<ToCowRcSlice<T>> {
	fn default() -> Self {
		pipeline!(Box::<[T]>::default() => Self::from)
	}
}

impl<T, Item> FromIterator<Item> for CowRc<[T]>
where
	Vec<T>: FromIterator<Item>,
{
	fn from_iter<Iterator: IntoIterator<Item = Item>>(iter: Iterator) -> Self {
		pipeline!(iter |> Vec::from_iter |> Self::from)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn deref_test() {
		let to_rc_slice: &ToCowRcSlice<i32> = ToCowRcSlice::from_slice(&[1, 2, 3]);
		let rc_slice: CowRc<[i32]> = to_rc_slice.to_owned();
		assert_eq!(rc_slice.len(), 3);
	}

	#[test]
	fn cow() {
		let cow: Cow<'_, ToCowRcSlice<i32>> =
			pipeline!(&[1, 2, 3][..] => CowRc::from => Cow::Owned);
		// Cow deref sur ToCowRcStr qui deref sur [T]
		assert_eq!(cow.len(), 3); // Le double deref a bien marché
	}
}
