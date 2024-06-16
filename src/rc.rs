use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	rc::{Rc, Weak},
};

use sugaru::pipeline;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct CowRc<T: ?Sized> {
	pub rc: Rc<T>,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct WeakCowRc<T: ?Sized> {
	pub weak: Weak<T>,
}

impl<T> CowRc<T> {
	pub fn new(value: T) -> Self {
		pipeline!(value |> Rc::new |> Self::from_rc)
	}
}

impl<T: ?Sized> CowRc<T> {
	pub const fn from_rc(rc: Rc<T>) -> Self {
		Self { rc }
	}

	#[inline]
	pub fn needs_cloning_to_mutate(this: &Self) -> bool {
		pipeline!(&this.rc => Rc::strong_count) > 1
	}

	#[inline]
	pub fn is_unique(this: &Self) -> bool {
		!Self::needs_cloning_to_mutate(this) && Rc::weak_count(&this.rc) == 0
	}

	pub fn downgrade(this: &Self) -> WeakCowRc<T> {
		pipeline!(&this.rc => Rc::downgrade => WeakCowRc::from_weak)
	}
}

impl<T, U> From<T> for CowRc<U>
where
	U: ?Sized,
	Rc<U>: From<T>,
{
	fn from(value: T) -> Self {
		pipeline!(value |> Rc::from |> Self::from_rc)
	}
}

impl<T: ?Sized> AsRef<T> for CowRc<T> {
	fn as_ref(&self) -> &T {
		pipeline!(&self.rc => Rc::as_ref)
	}
}

impl<T: ?Sized + Clone> AsMut<T> for CowRc<T> {
	fn as_mut(&mut self) -> &mut T {
		// make_mut doit potentiellement cloner mais on accepte le coût
		pipeline!(&mut self.rc => Rc::make_mut)
	}
}

impl<T: ?Sized> Deref for CowRc<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		pipeline!(&self.rc => Rc::deref)
	}
}

impl<T: ?Sized + Clone> DerefMut for CowRc<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// makes implicit what was explicit
		self.as_mut()
	}
}

impl<T> WeakCowRc<T> {
	pub const fn new() -> Self {
		pipeline!(Weak::new() => Self::from_weak)
	}
}

impl<T: ?Sized> WeakCowRc<T> {
	pub const fn from_weak(weak: Weak<T>) -> Self {
		Self { weak }
	}

	#[must_use = "this returns a new `CowRc`, without modifying the original weak pointer"]
	pub fn upgrade(this: &Self) -> Option<CowRc<T>> {
		pipeline!(&this.weak => Weak::upgrade).map(CowRc::from_rc)
	}
}

#[cfg(test)]
mod tests {
	use crate::rc::CowRc;

	#[derive(Debug, Clone)]
	struct Person {
		age: u8, // data small enough to be passed by value rather than reference
		purse: CowRc<Purse>,
	}

	// Let's pretend it's a big struct
	#[derive(Debug, Clone)]
	struct Purse {
		nb_of_keys: u8,
	}

	#[test]
	fn sandbox() {
		let person1 = Person {
			age: 46,
			purse: CowRc::new(Purse { nb_of_keys: 4 }),
		};

		let mut person2 = Person {
			age: 35,
			purse: person1.purse.clone(), // Just a strong count increment
		};

		// Optimistic in place mutation, purse is in reality cloned
		person2.purse.nb_of_keys -= 1; // Oops, lost a key

		assert_eq!(person1.purse.nb_of_keys, 4); // Original person is unaffected
		assert_eq!(person2.purse.nb_of_keys, 3);
	}
}
