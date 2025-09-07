//! Merkle Sum Tree implementation with MiMC hash function for zero-knowledge proofs.

mod constants;
mod mimc_sponge;

use crate::mimc_sponge::{Fr, MimcSponge};
use anyhow::Result;
use ff::{self, *};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Clone, PartialEq)]
pub enum MerkleError {
    IndexOutOfBounds { index: usize, max: usize },
    EmptyTree,
    InvalidLeaf(String),
    HashError(String),
    InvalidProof,
    OverflowError,
    InvalidTree(String),
}

impl fmt::Display for MerkleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MerkleError::IndexOutOfBounds { index, max } => {
                write!(f, "Index {} out of bounds, max: {}", index, max)
            }
            MerkleError::EmptyTree => write!(f, "Operation not allowed on empty tree"),
            MerkleError::InvalidLeaf(msg) => write!(f, "Invalid leaf: {}", msg),
            MerkleError::HashError(msg) => write!(f, "Hash error: {}", msg),
            MerkleError::InvalidProof => write!(f, "Proof verification failed"),
            MerkleError::OverflowError => write!(f, "Integer overflow in sum calculation"),
            MerkleError::InvalidTree(msg) => write!(f, "Invalid tree: {}", msg),
        }
    }
}

impl std::error::Error for MerkleError {}

#[derive(Debug)]
pub struct MerkleSumTree {
    leafs: Vec<Leaf>,
    nodes: Vec<Node>,
    height: usize,
    zero_index: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    id: String,
    node: Node,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    hash: Fr,
    value: i32,
}

#[derive(Debug, PartialEq)]
pub struct InclusionProof {
    leaf: Leaf,
    path: Vec<Neighbor>,
}

#[derive(Debug, PartialEq)]
pub struct Neighbor {
    position: Position,
    node: Node,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Left,
    Right,
}

impl Node {
    pub fn new(hash: Fr, value: i32) -> Node {
        Node { hash, value }
    }
    pub fn get_hash(&self) -> Fr {
        self.hash
    }
    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn is_equal(&self, node: Node) -> bool {
        self.get_hash() == node.get_hash() && self.get_value() == node.get_value()
    }
}

impl Leaf {
    pub fn new(id: String, value: i32) -> Leaf {
        let mut hr = DefaultHasher::new();
        id.hash(&mut hr);
        let hash = Fr::from_u128(hr.finish() as u128);
        let node = Node { hash, value };
        Leaf { id, node }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_node(&self) -> Node {
        self.node
    }

    pub fn is_none(&self) -> bool {
        self.get_id() == "0" && self.get_node().get_value() == 0
    }
}

impl Neighbor {
    pub fn new(position: Position, node: Node) -> Neighbor {
        Neighbor { position, node }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }
    pub fn get_node(&self) -> Node {
        self.node
    }
}

impl MerkleSumTree {
    pub fn new(leafs: Vec<Leaf>) -> Result<MerkleSumTree, MerkleError> {
        Self::create_tree(leafs)
    }

    pub fn get_root_hash(&self) -> Result<Fr, MerkleError> {
        match self.nodes.len() {
            0 => Err(MerkleError::EmptyTree),
            n => Ok(self.nodes[n - 1].get_hash()),
        }
    }

    pub fn get_root_sum(&self) -> Result<i32, MerkleError> {
        match self.nodes.len() {
            0 => Err(MerkleError::EmptyTree),
            n => Ok(self.nodes[n - 1].get_value()),
        }
    }

    pub fn get_root(&self) -> Result<Node, MerkleError> {
        match self.nodes.len() {
            0 => Err(MerkleError::EmptyTree),
            n => self.get_node(n - 1),
        }
    }

    pub fn get_nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn get_leafs(&self) -> &[Leaf] {
        &self.leafs
    }

    pub fn get_zero_index(&self) -> &[usize] {
        &self.zero_index
    }

    pub fn get_node(&self, index: usize) -> Result<Node, MerkleError> {
        if index >= self.nodes.len() {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                max: self.nodes.len().saturating_sub(1),
            });
        }
        Ok(self.nodes[index])
    }

    pub fn get_leaf(&self, index: usize) -> Result<Leaf, MerkleError> {
        if index >= self.leafs.len() {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                max: self.leafs.len().saturating_sub(1),
            });
        }
        Ok(self.leafs[index].clone())
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    fn update_path(&mut self, leaf: Leaf, index: usize) -> Result<(), MerkleError> {
        let height = self.height;
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        let mut current_node = leaf.get_node();
        for _ in 1..height {
            if current_index % 2 == 0 {
                let neighbor = self.get_node(current_index + 1)?;
                current_node = Self::build_parent(current_node, neighbor)?;
            } else {
                let neighbor = self.get_node(current_index - 1)?;
                current_node = Self::build_parent(neighbor, current_node)?;
            }
            level_start += level_size;
            level_index /= 2;
            current_index = level_start + level_index;
            level_size /= 2;
            self.nodes[current_index] = current_node;
        }
        Ok(())
    }

    pub fn get_proof(&self, index: usize) -> Result<InclusionProof, MerkleError> {
        if index >= self.get_leafs().len() {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                max: self.leafs.len().saturating_sub(1),
            });
        }
        let leaf = self.get_leaf(index)?;
        let mut path = vec![];
        let height = self.get_height();
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        for _ in 1..height {
            if current_index % 2 == 0 {
                let node = self.get_node(current_index + 1)?;
                let neighbor = Neighbor {
                    position: Position::Right,
                    node,
                };
                path.push(neighbor);
            } else {
                let node = self.get_node(current_index - 1)?;
                let neighbor = Neighbor {
                    position: Position::Left,
                    node,
                };
                path.push(neighbor);
            }
            level_start += level_size;
            level_index /= 2;
            current_index = level_start + level_index;
            level_size /= 2;
        }
        Ok(InclusionProof { leaf, path })
    }

    pub fn verify_proof(&self, proof: &InclusionProof) -> Result<bool, MerkleError> {
        let mut node = proof.leaf.get_node();

        for neighbor in proof.get_path() {
            match neighbor.position {
                Position::Right => node = Self::build_parent(node, neighbor.node)?,
                Position::Left => node = Self::build_parent(neighbor.node, node)?,
            }
        }
        let root = self.get_root()?;
        Ok(node.is_equal(root))
    }

    fn create_tree(mut leafs: Vec<Leaf>) -> Result<MerkleSumTree, MerkleError> {
        if leafs.is_empty() {
            return Err(MerkleError::InvalidTree(
                "Cannot create tree with no leaves".to_string(),
            ));
        }

        let (height, mut zero_index) = Self::fill_leafs(&mut leafs)?;

        let mut nodes: Vec<Node> = vec![];
        let mut nodes_to_hash: Vec<Node> = vec![];
        let mut temp_hash_nodes: Vec<Node> = vec![];

        for (i, leaf) in leafs.iter().enumerate() {
            if leaf.is_none() {
                zero_index.push(i)
            }
            let node = leaf.get_node();
            nodes.push(node);
            nodes_to_hash.push(node);
        }

        while nodes_to_hash.len() > 1 {
            let mut j = 0;
            while j < nodes_to_hash.len() {
                let new_node = Self::build_parent(nodes_to_hash[j], nodes_to_hash[j + 1])?;
                nodes.push(new_node);
                temp_hash_nodes.push(new_node);
                j += 2;
            }
            nodes_to_hash = std::mem::take(&mut temp_hash_nodes);
        }

        Ok(MerkleSumTree {
            leafs,
            nodes,
            height,
            zero_index,
        })
    }

    fn fill_leafs(leafs: &mut Vec<Leaf>) -> Result<(usize, Vec<usize>), MerkleError> {
        if leafs.is_empty() {
            return Err(MerkleError::InvalidLeaf(
                "Cannot process empty leaf vector".to_string(),
            ));
        }

        let mut power = 1;
        let mut height = 1;
        let mut zero_index = vec![];

        while power < leafs.len() {
            power <<= 1;
            height += 1;
            if height > 64 {
                return Err(MerkleError::InvalidTree("Tree too large".to_string()));
            }
        }

        let mut index = leafs.len();
        let empty_leaf = Leaf::new("0".to_string(), 0);
        let fill_count = power - leafs.len();
        leafs.reserve(fill_count);
        for _ in 0..fill_count {
            zero_index.push(index);
            leafs.push(empty_leaf.clone());
            index += 1;
        }
        Ok((height, zero_index))
    }

    fn build_parent(child_1: Node, child_2: Node) -> Result<Node, MerkleError> {
        let sum = child_1
            .get_value()
            .checked_add(child_2.get_value())
            .ok_or(MerkleError::OverflowError)?;

        let child_1_value_fr =
            Fr::from_str_vartime(&child_1.get_value().to_string()).ok_or_else(|| {
                MerkleError::HashError("Failed to convert child_1 value to Fr".to_string())
            })?;
        let child_2_value_fr =
            Fr::from_str_vartime(&child_2.get_value().to_string()).ok_or_else(|| {
                MerkleError::HashError("Failed to convert child_2 value to Fr".to_string())
            })?;
        let k = Fr::from_str_vartime("0")
            .ok_or_else(|| MerkleError::HashError("Failed to create zero Fr".to_string()))?;

        let arr = vec![
            child_1.get_hash(),
            child_1_value_fr,
            child_2.get_hash(),
            child_2_value_fr,
        ];

        let ms = MimcSponge::default();
        let hash = ms.multi_hash(&arr, k, 1);

        if hash.is_empty() {
            return Err(MerkleError::HashError(
                "Hash computation returned empty result".to_string(),
            ));
        }

        Ok(Node::new(hash[0], sum))
    }

    pub fn push(&mut self, leaf: Leaf) -> Result<usize, MerkleError> {
        match self.zero_index.len() {
            0 => {
                let index_value = self.leafs.len();
                self.leafs.push(leaf);
                let new_tree = Self::create_tree(self.leafs.clone())?;
                self.update_tree(new_tree)?;
                Ok(index_value)
            }
            _ => {
                let index_value = self.zero_index[0];
                self.set_leaf(leaf, index_value)?;
                Ok(index_value)
            }
        }
    }

    pub fn set_leaf(&mut self, leaf: Leaf, index: usize) -> Result<(), MerkleError> {
        if index >= self.leafs.len() {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                max: self.leafs.len().saturating_sub(1),
            });
        }

        let current_leaf = self.get_leaf(index)?;

        if leaf.is_none() && !current_leaf.is_none() {
            let pos = self.zero_index.binary_search(&index).unwrap_or_else(|e| e);
            self.zero_index.insert(pos, index);
        } else if !leaf.is_none() && current_leaf.is_none() {
            if let Ok(pos) = self.zero_index.binary_search(&index) {
                self.zero_index.remove(pos);
            }
        }

        self.leafs[index] = leaf.clone();
        self.nodes[index] = leaf.get_node();
        self.update_path(leaf, index)?;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<(), MerkleError> {
        if index >= self.leafs.len() {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                max: self.leafs.len().saturating_sub(1),
            });
        }
        let leaf = Leaf::new("0".to_string(), 0);
        self.set_leaf(leaf, index)?;
        Ok(())
    }

    fn update_tree(&mut self, tree: MerkleSumTree) -> Result<(), MerkleError> {
        self.leafs = tree.leafs;
        self.nodes = tree.nodes;
        self.height = tree.height;
        self.zero_index = tree.zero_index;
        Ok(())
    }
}

impl InclusionProof {
    pub fn get_path(&self) -> &[Neighbor] {
        &self.path
    }
    pub fn get_leaf(&self) -> &Leaf {
        &self.leaf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tree_creation() {
        let leaf_1 = Leaf::new("user1".to_string(), 100);
        let leaf_2 = Leaf::new("user2".to_string(), 200);
        let leafs = vec![leaf_1, leaf_2];

        let tree = MerkleSumTree::new(leafs).expect("Failed to create tree");

        assert_eq!(tree.get_height(), 2);
        assert!(tree.get_root_sum().is_ok());
        assert_eq!(tree.get_root_sum().unwrap(), 300);
    }

    #[test]
    fn test_empty_tree_error() {
        let leafs: Vec<Leaf> = vec![];
        let result = MerkleSumTree::new(leafs);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MerkleError::InvalidTree(_)));
    }

    #[test]
    fn test_inclusion_proof_generation_and_verification() {
        let leaf_1 = Leaf::new("account1".to_string(), 100);
        let leaf_2 = Leaf::new("account2".to_string(), 200);
        let leaf_3 = Leaf::new("account3".to_string(), 150);
        let leaf_4 = Leaf::new("account4".to_string(), 75);

        let leafs = vec![leaf_1.clone(), leaf_2, leaf_3, leaf_4];
        let tree = MerkleSumTree::new(leafs).expect("Failed to create tree");

        let proof = tree.get_proof(0).expect("Failed to generate proof");

        let is_valid = tree.verify_proof(&proof).expect("Failed to verify proof");
        assert!(is_valid, "Proof should be valid");

        assert_eq!(proof.get_leaf().get_id(), "account1");
        assert_eq!(proof.get_leaf().get_node().get_value(), 100);
    }

    #[test]
    fn test_index_out_of_bounds() {
        let leafs = vec![Leaf::new("test".to_string(), 1)];
        let tree = MerkleSumTree::new(leafs).unwrap();

        let result = tree.get_proof(10);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MerkleError::IndexOutOfBounds { .. }
        ));

        let result = tree.get_leaf(10);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MerkleError::IndexOutOfBounds { .. }
        ));
    }

    #[test]
    fn test_tree_operations() {
        let leaf_1 = Leaf::new("user1".to_string(), 50);
        let leaf_2 = Leaf::new("user2".to_string(), 100);
        let leafs = vec![leaf_1, leaf_2];

        let mut tree = MerkleSumTree::new(leafs).expect("Failed to create tree");
        let initial_sum = tree.get_root_sum().unwrap();
        assert_eq!(initial_sum, 150);

        let new_leaf = Leaf::new("user3".to_string(), 75);
        let index = tree.push(new_leaf).expect("Failed to push leaf");

        let updated_sum = tree.get_root_sum().unwrap();
        assert_eq!(updated_sum, 225);

        tree.remove(index).expect("Failed to remove leaf");
        let final_sum = tree.get_root_sum().unwrap();
        assert_eq!(final_sum, 150);
    }

    #[test]
    fn test_node_equality_constant_time() {
        let node1 = Node::new(Fr::from_str_vartime("123").expect("Valid Fr"), 100);
        let node2 = Node::new(Fr::from_str_vartime("123").expect("Valid Fr"), 100);
        let node3 = Node::new(Fr::from_str_vartime("456").expect("Valid Fr"), 200);

        assert!(node1.is_equal(node2));
        assert!(!node1.is_equal(node3));
    }

    #[test]
    fn test_overflow_protection() {
        let leaf1 = Leaf::new("test1".to_string(), i32::MAX - 1);
        let leaf2 = Leaf::new("test2".to_string(), 2);

        let result = MerkleSumTree::build_parent(leaf1.get_node(), leaf2.get_node());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MerkleError::OverflowError));
    }

    #[test]
    fn test_get_methods_return_references() {
        let leafs = vec![Leaf::new("test".to_string(), 1)];
        let tree = MerkleSumTree::new(leafs).unwrap();

        let nodes_ref = tree.get_nodes();
        let leafs_ref = tree.get_leafs();
        let zero_index_ref = tree.get_zero_index();

        assert!(!nodes_ref.is_empty());
        assert!(!leafs_ref.is_empty());
        let _ = zero_index_ref.len();
    }
}
