<div align="center"><br><br>

Smart references to your graph nodes.

<img src="logo.png" alt="internode" width="200">

[![crates.io](https://img.shields.io/crates/v/internode)](https://crates.io/crates/internode) [![docs.rs](https://img.shields.io/docsrs/internode)](https://docs.rs/internode/latest/internode/) [![License](https://img.shields.io/github/license/yuhr/internode)](https://github.com/yuhr/internode/blob/develop/LICENSE)

<br><br></div>

`internode` provides a hassle-free way to manage ownership of graph structures.

## Features

- Ownership propagation to connected nodes
- Leak-free cyclic references
- Customizable node implementation
- Thread-safety
- Do One Thing and Do It Well™︎
- Almost zero dependency (only [`genawaiter`](https://crates.io/crates/genawaiter) until [generators](https://github.com/rust-lang/rust/issues/43122) stabilize)

## Usage

At first, define the shape of your nodes. In this example, we'll use a simple implementation that holds a pair of lists that represents in- and out-neighbors. The important point here is, **inter-node references should be held through `Internode`**:

```rust
use internode::*;

#[derive(Default)]
struct Entity {
	succs: Vec<Internode<Entity>>,
	preds: Vec<Internode<Entity>>,
}

impl Entity {
	fn add_edge(from: &Internode<Entity>, to: &Internode<Entity>) {
		// Accessing the content of node is done by `lock`.
		from.lock().unwrap().succs.push(to.clone());
		to.lock().unwrap().preds.push(from.clone());
	}
}

impl Neighbors for Entity {
	type Iter<'a> = std::iter::Cloned<std::slice::Iter<'a, Internode<Entity>>>;
	fn outgoing(&self) -> Self::Iter<'_> { self.succs.iter().cloned() }
	fn incoming(&self) -> Self::Iter<'_> { self.preds.iter().cloned() }
}
```

Then, create nodes by `Node::new`:

```rust
let (a_weak, b) = {
	// `Node` is owning reference, while `Internode` is non-owning.
	let a = Node::new(Entity::default());
	let b = Node::new(Entity::default());
	// `Node` implements `Deref` to `Internode`.
	Entity::add_edge(&*a, &*b);
	// Downgrading `Node` yields `Internode`.
	let a_weak = a.downgrade();
	(a_weak, b)
	// `a` is dropped here.
};
// Upgrading `Internode` yields `Option<Node>`.
assert!(a_weak.upgrade().is_some());
// Dropping the last owning reference to the graph drops all nodes.
drop(b);
assert!(a_weak.upgrade().is_none());
```

### Cyclic Graphs

Cyclic references do work out of the box without memory leaks:

```rust
let (a_weak, b_weak, c_weak, c) = {
	let a = Node::new(Entity::default());
	let b = Node::new(Entity::default());
	let c = Node::new(Entity::default());
	Entity::add_edge(&*a, &*b);
	Entity::add_edge(&*b, &*c);
	Entity::add_edge(&*c, &*a);
	let a_weak = a.downgrade();
	let b_weak = b.downgrade();
	let c_weak = c.downgrade();
	(a_weak, b_weak, c_weak, c)
	// `a` and `b` are dropped here.
};
assert!(a_weak.upgrade().is_some());
assert!(b_weak.upgrade().is_some());
assert!(c_weak.upgrade().is_some());
drop(c);
assert!(a_weak.upgrade().is_none());
assert!(b_weak.upgrade().is_none());
assert!(c_weak.upgrade().is_none());
```

So you don't need to scratch your head about managing cycles anymore.

### Node Traversal

As a bonus for implementing `Neighbors`, methods to perform depth- and breadth-first searching are provided:

```rust
let a = Node::new(Entity::default());
let b = Node::new(Entity::default());
let c = Node::new(Entity::default());
let d = Node::new(Entity::default());
Entity::add_edge(&*a, &*b);
Entity::add_edge(&*a, &*c);
Entity::add_edge(&*b, &*d);
Entity::add_edge(&*c, &*d);
Entity::add_edge(&*d, &*a);
assert!(a.dfs_outgoing().eq([&*a, &*b, &*d, &*c].into_iter().cloned()));
assert!(a.dfs_incoming().eq([&*a, &*d, &*b, &*c].into_iter().cloned()));
assert!(a.bfs_outgoing().eq([&*a, &*b, &*c, &*d].into_iter().cloned()));
assert!(a.bfs_incoming().eq([&*a, &*d, &*b, &*c].into_iter().cloned()));
```

## Design Consideration

This crate is inspired by [`dendron`](https://crates.io/crates/dendron) (especially the concept that “reference to any node preserves entire tree”), which is limited to tree structures to ensure good properties, has a bunch of useful methods to manipulate, and also has defensive programming features like freezing nodes against edits. Such advanced functionalities are out of scope of `internode` and left to users, since requirements vary. For example, if you want your nodes to be frozen, then [`frozen`](https://crates.io/crates/frozen) or [more stringent implementation](https://users.rust-lang.org/t/immutable-frozen-t-type/23868) is nice to have.