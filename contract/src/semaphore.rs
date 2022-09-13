use crate::semaphore::field::*;
mod field;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    iter::{once, repeat, successors},
};


/// Hash types, values and algorithms for a Merkle tree
pub trait Hasher {
    /// Type of the leaf and node hashes
    type Hash: Clone + Eq + Serialize;

    /// Compute the hash of an intermediate node
    fn hash_node(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
}

/// Merkle tree with all leaf and intermediate hashes stored
#[allow(clippy::derive_partial_eq_without_eq)] // False positive
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MerkleTree<H: Hasher> {
    /// Depth of the tree, # of layers including leaf layer
    depth: usize,

    /// Hash value of empty subtrees of given depth, starting at leaf level
    empty: Vec<H::Hash>,

    /// Hash values of tree nodes and leaves, breadth first order
    nodes: Vec<H::Hash>,
}

/// Element of a Merkle proof
#[allow(clippy::derive_partial_eq_without_eq)] // False positive
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Branch<H: Hasher> {
    /// Left branch taken, value is the right sibling hash.
    Left(H::Hash),

    /// Right branch taken, value is the left sibling hash.
    Right(H::Hash),
}

/// Merkle proof path, bottom to top.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct Proof<H: Hasher>(pub Vec<Branch<H>>);

/// For a given node index, return the parent node index
/// Returns None if there is no parent (root node)
const fn parent(index: usize) -> Option<usize> {
    if index == 0 {
        None
    } else {
        Some(((index + 1) >> 1) - 1)
    }
}

/// For a given node index, return index of the first (left) child.
const fn first_child(index: usize) -> usize {
    (index << 1) + 1
}

const fn depth(index: usize) -> usize {
    // `n.next_power_of_two()` will return `n` iff `n` is a power of two.
    // The extra offset corrects this.
    (index + 2).next_power_of_two().trailing_zeros() as usize - 1
}

impl<H: Hasher> MerkleTree<H> {
    /// Creates a new `MerkleTree`
    /// * `depth` - The depth of the tree, including the root. This is 1 greater
    ///   than the `treeLevels` argument to the Semaphore contract.
    pub fn new(depth: usize, initial_leaf: H::Hash) -> Self {
        // Compute empty node values, leaf to root
        let empty = successors(Some(initial_leaf), |prev| Some(H::hash_node(prev, prev)))
            .take(depth)
            .collect::<Vec<_>>();

        // Compute node values
        let nodes = empty
            .iter()
            .rev()
            .enumerate()
            .flat_map(|(depth, hash)| repeat(hash).take(1 << depth))
            .cloned()
            .collect::<Vec<_>>();
        debug_assert!(nodes.len() == (1 << depth) - 1);

        Self {
            depth,
            empty,
            nodes,
        }
    }

    #[must_use]
    pub fn num_leaves(&self) -> usize {
        self.depth
            .checked_sub(1)
            .map(|n| 1 << n)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn root(&self) -> H::Hash {
        self.nodes[0].clone()
    }

    pub fn set(&mut self, leaf: usize, hash: H::Hash) {
        self.set_range(leaf, once(hash));
    }

    pub fn set_range<I: IntoIterator<Item = H::Hash>>(&mut self, start: usize, hashes: I) {
        let index = self.num_leaves() + start - 1;
        let mut count = 0;
        // TODO: Error/panic when hashes is longer than available leafs
        for (leaf, hash) in self.nodes[index..].iter_mut().zip(hashes) {
            *leaf = hash;
            count += 1;
        }
        if count != 0 {
            self.update_nodes(index, index + (count - 1));
        }
    }

    fn update_nodes(&mut self, start: usize, end: usize) {
        debug_assert_eq!(depth(start), depth(end));
        if let (Some(start), Some(end)) = (parent(start), parent(end)) {
            for parent in start..=end {
                let child = first_child(parent);
                self.nodes[parent] = H::hash_node(&self.nodes[child], &self.nodes[child + 1]);
            }
            self.update_nodes(start, end);
        }
    }

    #[must_use]
    pub fn proof(&self, leaf: usize) -> Option<Proof<H>> {
        if leaf >= self.num_leaves() {
            return None;
        }
        let mut index = self.num_leaves() + leaf - 1;
        let mut path = Vec::with_capacity(self.depth);
        while let Some(parent) = parent(index) {
            // Add proof for node at index to parent
            path.push(match index & 1 {
                1 => Branch::Left(self.nodes[index + 1].clone()),
                0 => Branch::Right(self.nodes[index - 1].clone()),
                _ => unreachable!(),
            });
            index = parent;
        }
        Some(Proof(path))
    }

    #[must_use]
    pub fn verify(&self, hash: H::Hash, proof: &Proof<H>) -> bool {
        proof.root(hash) == self.root()
    }

    #[must_use]
    pub fn leaves(&self) -> &[H::Hash] {
        &self.nodes[(self.num_leaves() - 1)..]
    }
}

impl<H: Hasher> Proof<H> {
    /// Compute the leaf index for this proof
    #[must_use]
    pub fn leaf_index(&self) -> usize {
        self.0.iter().rev().fold(0, |index, branch| match branch {
            Branch::Left(_) => index << 1,
            Branch::Right(_) => (index << 1) + 1,
        })
    }

    /// Compute path index (TODO: do we want to keep this here?)
    #[must_use]
    pub fn path_index(&self) -> Vec<Field> {
        self.0
            .iter()
            .map(|branch| match branch {
                Branch::Left(_) => Field::from(0),
                Branch::Right(_) => Field::from(1),
            })
            .collect()
    }

    /// Compute the Merkle root given a leaf hash
    #[must_use]
    pub fn root(&self, hash: H::Hash) -> H::Hash {
        self.0.iter().fold(hash, |hash, branch| match branch {
            Branch::Left(sibling) => H::hash_node(&hash, sibling),
            Branch::Right(sibling) => H::hash_node(sibling, &hash),
        })
    }
}

impl<H> Debug for Branch<H>
where
    H: Hasher,
    H::Hash: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(arg0) => f.debug_tuple("Left").field(arg0).finish(),
            Self::Right(arg0) => f.debug_tuple("Right").field(arg0).finish(),
        }
    }
}

impl<H> Debug for Proof<H>
where
    H: Hasher,
    H::Hash: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Proof").field(&self.0).finish()
    }
}