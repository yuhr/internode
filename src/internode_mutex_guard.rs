use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::NonNull;
use std::sync::MutexGuard;

use super::*;

/// A special mutex guard for the inner value of a node.
///
/// Implements [`Deref`] and [`DerefMut`], so users can think of this as just [`MutexGuard<T>`].
///
/// Returned by [`Node::value`] and [`Internode::lock`].
#[derive(Debug)]
pub struct InternodeMutexGuard<'a, T: Neighbors> {
	guard: MutexGuard<'a, Option<T>>,
}

impl<'a, T: Neighbors> InternodeMutexGuard<'a, T> {
	pub(crate) fn new(guard: MutexGuard<'a, Option<T>>) -> Self { Self { guard } }

	pub fn outgoing(self) -> impl 'a + Iterator<Item = Internode<T>> {
		InternodeMutexGuardIterOutgoing::new(self.guard)
	}

	pub fn incoming(self) -> impl 'a + Iterator<Item = Internode<T>> {
		InternodeMutexGuardIterIncoming::new(self.guard)
	}
}

impl<'a, T: Neighbors> Deref for InternodeMutexGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target { self.guard.as_ref().unwrap() }
}

impl<'a, T: Neighbors> DerefMut for InternodeMutexGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.guard.as_mut().unwrap() }
}

impl<'a, T: Neighbors + Display> Display for InternodeMutexGuard<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self.guard.as_ref().unwrap(), f)?;
		Ok(())
	}
}

struct InternodeMutexGuardIterOutgoing<'a, T: Neighbors> {
	guard: MutexGuard<'a, Option<T>>,
	iter: <T as Neighbors>::Iter<'a>,
}

impl<'a, T: Neighbors> InternodeMutexGuardIterOutgoing<'a, T> {
	pub fn new(mut guard: MutexGuard<'a, Option<T>>) -> Self {
		let value = unsafe { NonNull::new_unchecked(guard.as_mut().unwrap() as *mut T).as_ref() };
		let iter = value.outgoing();
		Self { guard, iter }
	}
}

impl<'a, T: Neighbors> Iterator for InternodeMutexGuardIterOutgoing<'a, T> {
	type Item = Internode<T>;
	fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
}

struct InternodeMutexGuardIterIncoming<'a, T: Neighbors> {
	guard: MutexGuard<'a, Option<T>>,
	iter: <T as Neighbors>::Iter<'a>,
}

impl<'a, T: Neighbors> InternodeMutexGuardIterIncoming<'a, T> {
	pub fn new(mut guard: MutexGuard<'a, Option<T>>) -> Self {
		let value = unsafe { NonNull::new_unchecked(guard.as_mut().unwrap() as *mut T).as_ref() };
		let iter = value.incoming();
		Self { guard, iter }
	}
}

impl<'a, T: Neighbors> Iterator for InternodeMutexGuardIterIncoming<'a, T> {
	type Item = Internode<T>;
	fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
}