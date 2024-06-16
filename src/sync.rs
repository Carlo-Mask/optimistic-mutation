use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	sync::{Arc, Weak},
};

use sugaru::pipeline;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct CowArc<T: ?Sized> {
	pub arc: Arc<T>,
}

#[derive(Debug, Clone, Default)]
pub struct WeakCowArc<T: ?Sized> {
	pub weak: Weak<T>,
}

impl<T> CowArc<T> {
	pub fn new(value: T) -> Self {
		pipeline!(value |> Arc::new |> Self::from_arc)
	}
}

impl<T: ?Sized> CowArc<T> {
	pub const fn from_arc(arc: Arc<T>) -> Self {
		Self { arc }
	}

	#[inline]
	pub fn needs_cloning_to_mutate(this: &Self) -> bool {
		pipeline!(&this.arc => Arc::strong_count) > 1
	}

	#[inline]
	pub fn is_unique(this: &Self) -> bool {
		!Self::needs_cloning_to_mutate(this) && Arc::weak_count(&this.arc) == 0
	}

	pub fn downgrade(this: &Self) -> WeakCowArc<T> {
		pipeline!(&this.arc => Arc::downgrade => WeakCowArc::from_weak)
	}
}

impl<T, U> From<T> for CowArc<U>
where
	U: ?Sized,
	Arc<U>: From<T>,
{
	fn from(value: T) -> Self {
		pipeline!(value |> Arc::from |> Self::from_arc)
	}
}

impl<T: ?Sized> AsRef<T> for CowArc<T> {
	fn as_ref(&self) -> &T {
		pipeline!(&self.arc => Arc::as_ref)
	}
}

impl<T: ?Sized + Clone> AsMut<T> for CowArc<T> {
	fn as_mut(&mut self) -> &mut T {
		pipeline!(&mut self.arc => Arc::make_mut)
	}
}

impl<T: ?Sized> Deref for CowArc<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		pipeline!(&self.arc => Arc::deref)
	}
}

impl<T: ?Sized + Clone> DerefMut for CowArc<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// makes implicit what was explicit
		self.as_mut()
	}
}

impl<T> WeakCowArc<T> {
	pub const fn new() -> Self {
		pipeline!(Weak::new() => Self::from_weak)
	}
}

impl<T: ?Sized> WeakCowArc<T> {
	pub const fn from_weak(weak: Weak<T>) -> Self {
		Self { weak }
	}

	#[must_use = "this returns a new `CowArc`, without modifying the original weak pointer"]
	pub fn upgrade(this: &Self) -> Option<CowArc<T>> {
		pipeline!(&this.weak => Weak::upgrade).map(CowArc::from_arc)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, Clone)]
	struct Person {
		age: u8, // data small enough to be passed by value rather than reference
		purse: CowArc<Purse>,
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
			purse: CowArc::new(Purse { nb_of_keys: 4 }),
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
