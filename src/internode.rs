use super::*;
use genawaiter::sync::Gen;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

#[derive(Default)]
struct InternodeImpl<T: Neighbors> {
	value: Mutex<Option<T>>,
	anchor: Mutex<Option<Weak<Anchor<T>>>>,
}

/// A non-owning shared reference to a node.
///
/// Returned by [`Node::downgrade`].
#[derive(Default)]
pub struct Internode<T: Neighbors>(Arc<InternodeImpl<T>>);

impl<T: Neighbors> Internode<T> {
	pub(crate) fn value(&self) -> &Mutex<Option<T>> { &self.0.value }

	pub(crate) fn anchor(&self) -> &Mutex<Option<Weak<Anchor<T>>>> { &self.0.anchor }

	pub(crate) fn new(value: T) -> Self {
		Self(Arc::new(InternodeImpl { value: Mutex::new(Some(value)), anchor: Mutex::new(None) }))
	}

	/// Blocks until the internal `Mutex` can be locked and returns a guard to the value. Will be `None` if this `Internode` is dropped already.
	pub fn lock(&self) -> Option<InternodeMutexGuard<'_, T>> {
		let guard = self.value().lock().unwrap();
		guard.is_some().then(|| InternodeMutexGuard::new(guard))
	}

	/// Tries to anchor this `Internode` into a `Node`.
	pub fn upgrade(&self) -> Option<Node<T>> {
		self.is_alive().then(|| Node::from_internode(self.clone()))
	}

	pub(crate) fn is_alive(&self) -> bool { self.value().lock().unwrap().is_some() }

	pub(crate) fn is_anchored(&self) -> bool { self.anchor().lock().unwrap().is_some() }

	pub(crate) fn anchor_upgraded(&self) -> Option<Arc<Anchor<T>>> {
		self.anchor().lock().unwrap().as_ref().and_then(Weak::upgrade)
	}

	pub(crate) fn should_live(&self) -> bool {
		std::iter::once(self.clone())
			.chain(self.dfs_incoming().skip(1))
			.chain(self.dfs_outgoing().skip(1))
			.any(|node| node.is_anchored())
	}

	pub(crate) fn release(&self) {
		let mut guard = self.value().lock().unwrap();
		if let Some(value) = guard.take() {
			drop(guard);
			for node in value.incoming().chain(value.outgoing()) {
				node.release()
			}
		}
	}

	/// Blocks until the internal `Mutex` can be locked and calls [`Neighbors::outgoing`].
	pub fn outgoing(&self) -> impl '_ + Iterator<Item = Self> {
		self.lock().into_iter().flat_map(InternodeMutexGuard::outgoing)
	}

	/// Blocks until the internal `Mutex` can be locked and calls [`Neighbors::incoming`].
	pub fn incoming(&self) -> impl '_ + Iterator<Item = Self> {
		self.lock().into_iter().flat_map(InternodeMutexGuard::incoming)
	}

	/// Performs a depth-first search by recursively calling [`Internode::outgoing`]. Includes the starting node first.
	pub fn dfs_outgoing(&self) -> impl '_ + Iterator<Item = Self> {
		Gen::new(|co| async move {
			let mut search = VecDeque::from([self.clone()]);
			let mut visited = HashSet::new();
			while let Some(node) = search.pop_front() {
				if visited.insert(node.clone()) {
					co.yield_(node.clone()).await;
					let len_old = search.len();
					search.extend(node.outgoing());
					search.rotate_left(len_old);
				}
			}
		})
		.into_iter()
	}

	/// Performs a depth-first search by recursively calling [`Internode::incoming`]. Includes the starting node first.
	pub fn dfs_incoming(&self) -> impl '_ + Iterator<Item = Self> {
		Gen::new(|co| async move {
			let mut search = VecDeque::from([self.clone()]);
			let mut visited = HashSet::new();
			while let Some(node) = search.pop_front() {
				if visited.insert(node.clone()) {
					co.yield_(node.clone()).await;
					let len_old = search.len();
					search.extend(node.incoming());
					search.rotate_left(len_old);
				}
			}
		})
		.into_iter()
	}

	/// Performs a breadth-first search by recursively calling [`Internode::outgoing`]. Includes the starting node first.
	pub fn bfs_outgoing(&self) -> impl '_ + Iterator<Item = Self> {
		Gen::new(|co| async move {
			let mut search = VecDeque::from([self.clone()]);
			let mut visited = HashSet::new();
			while let Some(node) = search.pop_front() {
				if visited.insert(node.clone()) {
					co.yield_(node.clone()).await;
					search.extend(node.outgoing());
				}
			}
		})
		.into_iter()
	}

	/// Performs a breadth-first search by recursively calling [`Internode::incoming`]. Includes the starting node first.
	pub fn bfs_incoming(&self) -> impl '_ + Iterator<Item = Self> {
		Gen::new(|co| async move {
			let mut search = VecDeque::from([self.clone()]);
			let mut visited = HashSet::new();
			while let Some(node) = search.pop_front() {
				if visited.insert(node.clone()) {
					co.yield_(node.clone()).await;
					search.extend(node.incoming());
				}
			}
		})
		.into_iter()
	}
}

impl<T: Neighbors> Clone for Internode<T> {
	fn clone(&self) -> Self { Self(Arc::clone(&self.0)) }
}

impl<T: Neighbors> PartialEq for Internode<T> {
	fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.0, &other.0) }
}

impl<T: Neighbors> Eq for Internode<T> {}

impl<T: Neighbors> Hash for Internode<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { Arc::as_ptr(&self.0).hash(state) }
}

impl<T: Neighbors + Debug> Debug for Internode<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Internode(")?;
		Debug::fmt(&*self.lock().unwrap(), f)?;
		write!(f, ")")?;
		Ok(())
	}
}

impl<T: Neighbors + Display> Display for Internode<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&*self.lock().unwrap(), f)?;
		Ok(())
	}
}