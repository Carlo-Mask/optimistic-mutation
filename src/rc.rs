use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	rc::{Rc, Weak},
};

use sugaru::pipeline;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct CowRc<T: ?Sized> {
	// Private to avoid name collision with a T containing a field named rc
	rc: Rc<T>,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct WeakCowRc<T: ?Sized> {
	pub weak: Weak<T>,
}

impl<T> CowRc<T> {
	/// Constructs a new `CowRc<T>`.
	///
	/// # Examples
	///
	/// ```
	/// use optimistic_mutation::rc::CowRc;
	///
	/// let five = CowRc::new(5);
	/// ```
	pub fn new(value: T) -> Self {
		pipeline!(value |> Rc::new |> Self::from_rc)
	}
}

#[allow(clippy::wrong_self_convention)] // CowRc est un smart pointer et il faut éviter les méthodes qui ont des noms
										// qui pourrait entrer en collision avec des méthodes de T
impl<T: ?Sized> CowRc<T> {
	/// Constructs a new `CowRc<T>` from a `Rc<T>`
	///
	/// # Examples
	///
	/// ```
	/// use std::rc::Rc;
	/// use optimistic_mutation::rc::CowRc;
	///
	/// let five = CowRc::from_rc(Rc::new(5));
	/// ```
	#[must_use]
	pub const fn from_rc(rc: Rc<T>) -> Self {
		Self { rc }
	}

	#[must_use]
	pub const fn as_rc(this: &Self) -> &Rc<T> {
		&this.rc
	}

	#[must_use]
	pub const fn as_rc_mut(this: &mut Self) -> &mut Rc<T> {
		&mut this.rc
	}

	#[must_use]
	pub fn unwrap_rc(this: Self) -> Rc<T> {
		this.rc
	}

	/// Return true if this `CowRc<T>` needs cloning to mutate
	/// cloning is necessary when the strong count is more than one
	/// but does not depend on the weak count
	#[inline]
	#[must_use]
	pub fn needs_cloning_to_mutate(this: &Self) -> bool {
		pipeline!(&this.rc => Rc::strong_count) > 1
	}

	#[inline]
	#[must_use]
	pub fn is_unique(this: &Self) -> bool {
		!Self::needs_cloning_to_mutate(this) && Rc::weak_count(&this.rc) == 0
	}

	#[must_use = "this returns a new `Weak` pointer, without modifying the original `CowRc`"]
	pub fn downgrade(this: &Self) -> WeakCowRc<T> {
		pipeline!(&this.rc => Rc::downgrade => WeakCowRc::from_weak)
	}
}

impl<T, U> From<T> for CowRc<U>
where
	U: ?Sized,
	Rc<U>: From<T>,
{
	/// Converts a generic type `T` into an `CowRc<T>`
	///
	/// The conversion accepts anything that can be turned [`Into`] an `Rc<T>`
	/// and produces a `CowRc<T>` containing a [`Rc`] created from `t`
	///
	/// # Example
	/// ```rust
	/// # use std::rc::Rc;
	/// # use optimistic_mutation::rc::CowRc;
	/// let x = 5;
	/// let  cow_rc = CowRc::from(x);
	///
	/// assert_eq!(cow_rc, CowRc::new(x));
	/// assert_eq!(CowRc::unwrap_rc(cow_rc), Rc::from(x));
	/// ```
	#[inline]
	fn from(value: T) -> Self {
		pipeline!(value |> Rc::from |> Self::from_rc)
	}
}

impl<T: ?Sized> Deref for CowRc<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		pipeline!(&self.rc => Rc::deref)
	}
}

impl<T: ?Sized + Clone> DerefMut for CowRc<T> {
	/// Mutably dereferences the given `CowRc` using `*cow_rc = 1` notation.
	///
	/// If there are other `CowRc` pointers to the same allocation, then `deref_mut` will
	/// [`clone`](Clone::clone) the inner value to a new allocation to ensure unique ownership.
	/// This is also referred to as clone-on-write.
	///
	/// Even though [`DerefMut`] is supposed to be used for cheap dereferencing,
	/// given the nature of `CowRc`, this performance hit is acceptable
	///
	/// If there are no other `CowRc` pointers to this allocation,
	/// then any existing [`WeakCowRc`] pointers will be disassociated and the inner value will not be cloned.
	/// This is referred to as optimistic mutation.
	///
	/// # Examples
	///
	/// ```
	/// use optimistic_mutation::rc::CowRc;
	///
	/// let mut data = CowRc::new(5);
	///
	/// *data += 1;         // Won't clone anything (optimistic mutation)
	/// let mut other_data = CowRc::clone(&data); // Won't clone inner data
	/// *data += 1;         // Clones inner data
	/// *data += 1;         // Won't clone anything
	/// *other_data *= 2;   // Won't clone anything
	///
	/// // Now `data` and `other_data` point to different allocations.
	/// assert_eq!(*data, 8);
	/// assert_eq!(*other_data, 12);
	/// ```
	///
	/// If there are [`WeakCowRc`] pointers, but no other strong pointer,
	/// no cloning occurs, but the data is moved and the weak pointers will be disassociated
	/// (as if the data was cloned and the old `CowRc` strong count dropped to 0):
	/// ```
	/// use optimistic_mutation::rc::CowRc;
	///
	/// let mut data = CowRc::new(75);
	/// let weak = CowRc::downgrade(&data);
	///
	/// assert_eq!(*data, 75);
	/// assert_eq!(*weak.upgrade().unwrap(), 75);
	///
	/// *data += 1;
	///
	/// assert_eq!(*data, 76);
	/// assert!(weak.upgrade().is_none());
	/// ```
	///
	/// However, if there are other strong references to prevent the strong count dropping to 0,
	/// cloning occurs and [`WeakCowRc`] pointers will continue to point to the old data
	/// ```
	/// use optimistic_mutation::rc::CowRc;
	///
	/// let mut data = CowRc::new(75);
	/// let clone = CowRc::clone(&data);
	/// let weak = CowRc::downgrade(&data);
	///
	/// assert_eq!(*weak.upgrade().unwrap(), 75);
	///
	/// *data += 1;
	///
	/// assert_eq!(*data, 76);
	/// assert_eq!(*weak.upgrade().unwrap(), 75);
	/// drop(clone);
	/// assert!(weak.upgrade().is_none())
	/// ```
	fn deref_mut(&mut self) -> &mut Self::Target {
		pipeline!(&mut self.rc => Rc::make_mut)
	}
}

impl<T: ?Sized> AsRef<T> for CowRc<T> {
	fn as_ref(&self) -> &T {
		pipeline!(&self.rc => Rc::as_ref)
	}
}

impl<T: ?Sized> AsRef<Rc<T>> for CowRc<T> {
	fn as_ref(&self) -> &Rc<T> {
		&self.rc
	}
}

impl<T: ?Sized + Clone> AsMut<T> for CowRc<T> {
	/// Makes a mutable reference into the given `CowRc`. Has the same effect as dereferencing it.
	///
	/// Even though [`AsMut`] is supposed to be used to do cheap mutable-to-mutable reference conversion,
	/// given the nature of `CowRc`, this performance hit is acceptable
	///
	/// See [`DerefMut` implementation for `CowRc`](CowRc::<T>::deref_mut) for details
	fn as_mut(&mut self) -> &mut T {
		self
	}
}

impl<T: ?Sized> AsMut<Rc<T>> for CowRc<T> {
	fn as_mut(&mut self) -> &mut Rc<T> {
		&mut self.rc
	}
}

impl<T> WeakCowRc<T> {
	#[must_use]
	pub const fn new() -> Self {
		pipeline!(Weak::new() => Self::from_weak)
	}
}

impl<T: ?Sized> WeakCowRc<T> {
	#[must_use]
	pub const fn from_weak(weak: Weak<T>) -> Self {
		Self { weak }
	}

	#[must_use = "this returns a new `CowRc`, without modifying the original weak pointer"]
	pub fn upgrade(&self) -> Option<CowRc<T>> {
		self.weak.upgrade().map(CowRc::from_rc)
	}
}

#[cfg(test)]
mod tests {
	use crate::rc::CowRc;
	use std::{ops::DerefMut, rc::Rc};
	use sugaru::pipeline;

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
		assert_eq!(person1.age, 46);
	}

	#[test]
	fn from_str() {
		let str_slice_reference: &str = "Hello";

		let cow_rc: CowRc<str> = CowRc::from(str_slice_reference);

		assert_eq!(&*cow_rc, str_slice_reference);
	}

	#[test]
	fn from_rc() {
		let rc = Rc::new(5);
		// Works because Rc<i32> implements From<Rc<i32>>
		let cow_rc: CowRc<i32> = CowRc::from(rc);

		assert_eq!(cow_rc.rc, Rc::new(5));
	}

	#[test]
	fn no_clone_when_unique() {
		#[derive(Debug)]
		struct TrapClone {
			int: i32,
		}

		impl Clone for TrapClone {
			fn clone(&self) -> Self {
				panic!("Test failed: clone was called")
			}
		}

		let mut unique_cow_rc = CowRc::new(TrapClone { int: 0 });
		unique_cow_rc.int += 1;
		unique_cow_rc.deref_mut().int += 1;

		assert_eq!(unique_cow_rc.int, 2);

		let weak_cow_rc = pipeline!(&unique_cow_rc => CowRc::downgrade);
		unique_cow_rc.int += 1;
		assert!(weak_cow_rc.upgrade().is_none());
	}
}
