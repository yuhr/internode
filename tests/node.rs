use internode::*;
use std::fmt::Debug;

struct Entity {
	value: &'static str,
	succs: Vec<Internode<Entity>>,
	preds: Vec<Internode<Entity>>,
}

impl Entity {
	fn new(value: &'static str) -> Self {
		Self { value, succs: Default::default(), preds: Default::default() }
	}

	fn add_edge(from: &Internode<Entity>, to: &Internode<Entity>) {
		from.lock().unwrap().succs.push(to.clone());
		to.lock().unwrap().preds.push(from.clone());
	}
}

impl Neighbors for Entity {
	type Iter<'a> = std::iter::Cloned<std::slice::Iter<'a, Internode<Entity>>>;
	fn outgoing(&self) -> Self::Iter<'_> { self.succs.iter().cloned() }
	fn incoming(&self) -> Self::Iter<'_> { self.preds.iter().cloned() }
}

impl Debug for Entity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let depth = f.precision().unwrap_or(0);
		if depth == 0 {
			write!(f, "Entity({}, ..)", self.value)?;
		} else {
			fn fmt_nodes<'a>(
				iter: impl 'a + Iterator<Item = &'a Internode<Entity>>,
				depth: usize,
			) -> String {
				iter.map(|node| format!("{node:.depth$?}, "))
					.collect::<String>()
					.trim_end_matches(", ")
					.to_string()
			}
			write!(
				f,
				"Entity({}, succs: [{}], preds: [{}])",
				self.value,
				fmt_nodes(self.succs.iter(), depth - 1),
				fmt_nodes(self.preds.iter(), depth - 1),
			)?;
		}
		Ok(())
	}
}

#[test]
fn lifecycle_0() {
	let (a_weak, b) = {
		let a = Node::new(Entity::new("a"));
		let b = Node::new(Entity::new("b"));
		Entity::add_edge(&*a, &*b);
		let a_weak = a.downgrade();
		(a_weak, b)
	};
	assert!(a_weak.upgrade().is_some());
	drop(b);
	assert!(a_weak.upgrade().is_none());
}

#[test]
fn lifecycle_1() {
	let (a_weak, b_weak, c_weak) = {
		let a = Node::new(Entity::new("a"));
		let b = Node::new(Entity::new("b"));
		let c = Node::new(Entity::new("c"));
		Entity::add_edge(&*a, &*b);
		Entity::add_edge(&*b, &*c);
		let a_weak = a.downgrade();
		let b_weak = b.downgrade();
		let c_weak = c.downgrade();
		(a_weak, b_weak, c_weak)
	};
	assert!(a_weak.upgrade().is_none());
	assert!(b_weak.upgrade().is_none());
	assert!(c_weak.upgrade().is_none());
}

#[test]
fn lifecycle_2() {
	let (a_weak, b_weak, c_weak, c) = {
		let a = Node::new(Entity::new("a"));
		let b = Node::new(Entity::new("b"));
		let c = Node::new(Entity::new("c"));
		Entity::add_edge(&*a, &*b);
		Entity::add_edge(&*b, &*c);
		Entity::add_edge(&*c, &*a);
		let a_weak = a.downgrade();
		let b_weak = b.downgrade();
		let c_weak = c.downgrade();
		(a_weak, b_weak, c_weak, c)
	};

	assert!(a_weak.upgrade().is_some());
	assert!(b_weak.upgrade().is_some());
	assert!(c_weak.upgrade().is_some());
	assert_eq!(format!("{b_weak:.1?}"), "Internode(Entity(b, succs: [Internode(Entity(c, ..))], preds: [Internode(Entity(a, ..))]))");
	assert_eq!(
		format!("{:.1?}", b_weak.upgrade().unwrap()),
		"Node(Entity(b, succs: [Internode(Entity(c, ..))], preds: [Internode(Entity(a, ..))]))"
	);
	drop(c);
	assert!(a_weak.upgrade().is_none());
	assert!(b_weak.upgrade().is_none());
	assert!(c_weak.upgrade().is_none());
}

#[test]
fn traversal() {
	let a = Node::new(Entity::new("a"));
	let b = Node::new(Entity::new("b"));
	let c = Node::new(Entity::new("c"));
	let d = Node::new(Entity::new("d"));
	Entity::add_edge(&*a, &*b);
	Entity::add_edge(&*a, &*c);
	Entity::add_edge(&*b, &*d);
	Entity::add_edge(&*c, &*d);
	Entity::add_edge(&*d, &*a);
	assert!(a.dfs_outgoing().eq([&*a, &*b, &*d, &*c].into_iter().cloned()));
	assert!(a.dfs_incoming().eq([&*a, &*d, &*b, &*c].into_iter().cloned()));
	assert!(a.bfs_outgoing().eq([&*a, &*b, &*c, &*d].into_iter().cloned()));
	assert!(a.bfs_incoming().eq([&*a, &*d, &*b, &*c].into_iter().cloned()));
}