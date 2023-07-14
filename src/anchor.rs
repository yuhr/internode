use super::*;
use std::sync::Arc;

pub(crate) struct Anchor<T: Neighbors> {
	inner: Internode<T>,
}

impl<T: Neighbors> Anchor<T> {
	pub(crate) fn new(inner: Internode<T>) -> Arc<Self> {
		if let Some(anchor) = inner.anchor_upgraded() {
			anchor
		} else {
			let anchor = Arc::new(Self { inner });
			anchor.inner.anchor().lock().unwrap().replace(Arc::downgrade(&anchor));
			anchor
		}
	}

	pub(crate) fn inner(&self) -> &Internode<T> { &self.inner }
}

impl<T: Neighbors> Drop for Anchor<T> {
	fn drop(&mut self) {
		drop(self.inner.anchor().lock().unwrap().take());
		if !self.inner.should_live() {
			self.inner.release()
		}
	}
}