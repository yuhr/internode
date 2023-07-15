use super::*;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

/// An owning shared reference to a node.
///
/// This “anchors” the entire connected graph to the memory. In order to avoid cyclic ownership i.e. memory leaks, `T` should **not** contain references to other nodes through this type. If dropped, and no other `Node`s to any single connected node are held elsewhere, the connected nodes will be dropped as well.
///
/// Returned by [`Node::new`] and [`Internode::upgrade`].
pub struct Node<T: Neighbors> {
	anchor: Arc<Anchor<T>>,
}

impl<T: Neighbors> Node<T> {
	pub(crate) fn from_internode(inner: Internode<T>) -> Self {
		Self { anchor: Anchor::new(inner) }
	}

	/// Creates a new `Node` with the given value.
	pub fn new(value: T) -> Self { Self::from_internode(Internode::new(value)) }

	/// Downgrades this `Node` into an `Internode`.
	pub fn downgrade(&self) -> Internode<T> { self.anchor.inner().clone() }

	/// Blocks until the internal `Mutex` can be locked and returns a guard to the value.
	pub fn lock(&self) -> InternodeMutexGuard<'_, T> { self.anchor.inner().lock().unwrap() }
}

impl<T: Neighbors> Deref for Node<T> {
	type Target = Internode<T>;
	fn deref(&self) -> &Self::Target { &self.anchor.inner() }
}

impl<T: Neighbors> Clone for Node<T> {
	fn clone(&self) -> Self { Self { anchor: Arc::clone(&self.anchor) } }
}

impl<T: Neighbors> PartialEq for Node<T> {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.anchor.inner(), &other.anchor.inner())
	}
}

impl<T: Neighbors> Eq for Node<T> {}

impl<T: Neighbors> Hash for Node<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { Hash::hash(&self.anchor.inner(), state); }
}

impl<T: Neighbors + Debug> Debug for Node<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Node(")?;
		Debug::fmt(&*self.lock(), f)?;
		write!(f, ")")?;
		Ok(())
	}
}

impl<T: Neighbors + Display> Display for Node<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&*self.lock(), f)?;
		Ok(())
	}
}