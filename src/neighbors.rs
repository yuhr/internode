use super::*;

/// Defines neighbors of a node.
pub trait Neighbors: Sized {
	type Iter<'a>: 'a + Iterator<Item = Internode<Self>>
	where Self: 'a;

	/// Returns an iterator over the outgoing neighbors of this node.
	fn outgoing(&self) -> Self::Iter<'_>;

	/// Returns an iterator over the incoming neighbors of this node.
	fn incoming(&self) -> Self::Iter<'_>;
}