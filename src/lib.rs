mod constants;
mod mimc_sponge;

use crate::mimc_sponge::{Fr, MimcSponge};
use anyhow::Result;
use ff::{self, *};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// TO ADD:
/// readME
/// Comments
/// Separate merkle and proof?
/// Borrowing, ownership, box?
/// Use of reference instead of clone each time?
///
/// LATER:
/// Hash function as input
/// Resize the tree
///

/// The list of nodes includes all nodes of the tree
/// It is in the form [h1, h2, h3, h4, h12, h34, h1234], where h1234 would be the root and h1-4 is a leaf.
/// The number of leafs needs to be power of 2. The tree will be filled with 0 to keep the power of 2 intact.
/// Leafs will return only the leafs without the 0 values.
/// Nodes will return all nodes inlcuding the 0 value leafs.
#[derive(Debug, Clone)]
pub struct MerkleSumTree {
    leafs: Vec<Leaf>,
    nodes: Vec<Node>,
    height: usize,
    zero_index: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Leaf {
    id: String,
    node: Node,
}

#[derive(Debug, Clone)]
pub struct Node {
    hash: Fr,
    value: i32,
}

#[derive(Debug, Clone)]
pub struct InclusionProof {
    leaf: Leaf,
    path: Vec<Neighbor>,
}

#[derive(Debug, Clone)]
pub struct Neighbor {
    position: Position,
    node: Node,
}

#[derive(Debug, Clone)]
pub enum Position {
    Left,
    Right,
}

impl Node {
    pub fn new(hash: Fr, value: i32) -> Node {
        Node { hash, value }
    }
    pub fn get_hash(&self) -> Fr {
        self.hash.clone()
    }
    pub fn get_value(&self) -> i32 {
        self.value.clone()
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

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_node(&self) -> Node {
        self.node.clone()
    }

    pub fn is_none(&self) -> bool {
        self.get_id() == "0".to_string() && self.get_node().get_value() == 0
    }
}

impl Neighbor {
    pub fn new(position: Position, node: Node) -> Neighbor {
        Neighbor { position, node }
    }

    pub fn get_position(&self) -> Position {
        self.position.clone()
    }
    pub fn get_node(&self) -> Node {
        self.node.clone()
    }
}

impl MerkleSumTree {
    pub fn new(leafs: Vec<Leaf>) -> Result<MerkleSumTree> {
        Self::create_tree(leafs)
    }

    pub fn get_root_hash(&self) -> Option<Fr> {
        match self.nodes.len() {
            0 => None,
            n => Some(self.nodes[n - 1].get_hash()),
        }
    }

    pub fn get_root_sum(&self) -> Option<i32> {
        match self.nodes.len() {
            0 => None,
            n => Some(self.nodes[n - 1].get_value()),
        }
    }

    pub fn get_root(&self) -> Option<Node> {
        match self.nodes.len() {
            0 => None,
            n => self.get_node(n - 1),
        }
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    pub fn get_leafs(&self) -> Vec<Leaf> {
        self.leafs.clone()
    }

    pub fn get_zero_index(&self) -> Vec<usize> {
        self.zero_index.clone()
    }

    pub fn get_node(&self, index: usize) -> Option<Node> {
        match index.cmp(&self.get_nodes().len()) {
            Ordering::Less => Some(self.nodes[index].clone()),
            _ => None,
        }
    }

    pub fn get_leaf(&self, index: usize) -> Option<Leaf> {
        match index.cmp(&self.get_leafs().len()) {
            Ordering::Less => Some(self.leafs[index].clone()),
            _ => None,
        }
    }

    pub fn get_height(&self) -> usize {
        self.height.clone()
    }

    fn update_path(&mut self, leaf: Leaf, index: usize) -> Result<()> {
        let height = self.height;
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        let mut current_node = leaf.get_node();
        for _ in 1..height {
            if current_index % 2 == 0 {
                let neighbor = self.get_node(current_index + 1).unwrap();
                current_node = Self::build_parent(current_node, neighbor)?;
            } else {
                let neighbor = self.get_node(current_index - 1).unwrap();
                current_node = Self::build_parent(neighbor, current_node)?;
            }
            level_start += level_size;
            level_index = level_index / 2;
            current_index = level_start + level_index;
            level_size = level_size / 2;
            self.nodes[current_index] = current_node.clone();
        }
        Ok(())
    }

    pub fn get_proof(&self, index: usize) -> Result<(Option<InclusionProof>)> {
        if self.get_leafs().len() <= index {
            return Ok(None);
        }
        let leaf = self.get_leaf(index).unwrap();
        let mut path = vec![];
        let height = self.get_height();
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        for _ in 1..height {
            if current_index % 2 == 0 {
                let node = self.get_node(current_index + 1).unwrap();
                let neighbor = Neighbor {
                    position: Position::Right,
                    node,
                };
                path.push(neighbor);
            } else {
                let node = self.get_node(current_index - 1).unwrap();
                let neighbor = Neighbor {
                    position: Position::Left,
                    node,
                };
                path.push(neighbor);
            }
            level_start += level_size;
            level_index = level_index / 2;
            current_index = level_start + level_index;
            level_size = level_size / 2;
        }
        Ok(Some(InclusionProof { leaf, path }))
    }

    pub fn verify_proof(&self, proof: InclusionProof) -> Result<bool> {
        let mut node = proof.leaf.get_node();
        let path = proof.path;

        for neighbor in path {
            match neighbor.position {
                Position::Right => {
                    node = Self::build_parent(node, neighbor.node)?;
                }
                Position::Left => {
                    node = Self::build_parent(neighbor.node, node)?;
                }
            }
        }
        Ok(node.is_equal(self.get_root().unwrap()))
    }

    fn create_tree(mut leafs: Vec<Leaf>) -> Result<MerkleSumTree> {
        let height;
        let mut zero_index = vec![];
        (leafs, height, zero_index) = Self::fill_leafs(leafs)?;
        let mut nodes: Vec<Node> = vec![];
        let mut nodes_to_hash: Vec<Node> = vec![];
        let mut temp_hash_nodes: Vec<Node> = vec![];
        for leaf in leafs.iter() {
            let node = leaf.get_node();
            nodes.push(node.clone());
            nodes_to_hash.push(node);
        }
        while nodes_to_hash.len() > 1 {
            let mut j = 0;
            while j < nodes_to_hash.len() {
                let new_node =
                    Self::build_parent(nodes_to_hash[j].clone(), nodes_to_hash[j + 1].clone())?;
                nodes.push(new_node.clone());
                temp_hash_nodes.push(new_node);
                j += 2;
            }
            nodes_to_hash = temp_hash_nodes.clone();
            temp_hash_nodes = vec![];
        }
        Ok(MerkleSumTree {
            leafs,
            nodes,
            height,
            zero_index,
        })
    }

    fn fill_leafs(mut leafs: Vec<Leaf>) -> Result<(Vec<Leaf>, usize, Vec<usize>)> {
        let mut power = 1;
        let mut height = 1;
        let mut zero_index = vec![];
        while power < leafs.len() {
            power = power << 1;
            height += 1
        }
        let mut index = leafs.len();
        let empty_leaf = Leaf::new("0".to_string(), 0);
        for _ in 0..power - leafs.len() {
            zero_index.push(index);
            leafs.push(empty_leaf.clone());
            index += 1;
        }
        Ok((leafs, height, zero_index))
    }

    fn build_parent(child_1: Node, child_2: Node) -> Result<Node> {
        let arr = vec![
            child_1.get_hash(),
            Fr::from_str_vartime(&child_1.get_value().to_string()).unwrap(),
            child_2.get_hash(),
            Fr::from_str_vartime(&child_2.get_value().to_string()).unwrap(),
        ];
        let k = Fr::from_str_vartime("0").unwrap();
        let ms = MimcSponge::default();
        let hash = ms.multi_hash(&arr, k, 1);
        let sum = child_1.get_value() + child_2.get_value();
        Ok(Node::new(hash[0], sum))
    }

    //Push new leaf, return index
    pub fn push(&mut self, leaf: Leaf) -> Result<usize> {
        match self.zero_index.len() {
            0 => {
                self.leafs.push(leaf);
                let index_value = self.leafs.len();
                let new_tree = Self::create_tree(self.leafs.clone())?;
                _ = self.update_tree(new_tree);
                Ok(index_value)
            }
            _ => {
                let index_value = self.zero_index[0];
                _ = self.set_leaf(leaf, index_value);
                Ok(index_value)
            }
        }
    }
    pub fn set_leaf(&mut self, leaf: Leaf, index: usize) -> Result<()> {
        if leaf.is_none() && !self.get_leaf(index).unwrap().is_none() {
            let pos = self.zero_index.binary_search(&index).unwrap_or_else(|e| e);
            self.zero_index.insert(pos, index);
        } else if !leaf.is_none() && self.get_leaf(index).unwrap().is_none() {
            let pos = self.zero_index.binary_search(&index).unwrap_or_else(|e| e);
            self.zero_index.remove(pos);
        }
        self.leafs[index] = leaf.clone();
        self.nodes[index] = leaf.get_node();
        _ = self.update_path(leaf, index);
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        let leaf = Leaf::new("0".to_string(), 0);
        _ = self.set_leaf(leaf, index);
        Ok(())
    }

    fn update_tree(&mut self, tree: MerkleSumTree) -> Result<()> {
        self.leafs = tree.leafs;
        self.nodes = tree.nodes;
        self.height = tree.height;
        self.zero_index = tree.zero_index;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn it_works() {
        let leaf_1 = Leaf::new("11672136".to_string(), 10);
        let leaf_2 = Leaf::new("10566265".to_string(), 11);
        let leaf_3 = Leaf::new("10566215".to_string(), 12);
        let leaf_4 = Leaf::new("10566215".to_string(), 13);
        let leaf_5 = Leaf::new("10566215".to_string(), 14);
        let leaf_0 = Leaf::new("0".to_string(), 0);
        let mut merkle_sum_tree = MerkleSumTree::new(vec![leaf_1]).unwrap();
        merkle_sum_tree.push(leaf_2);
        merkle_sum_tree.push(leaf_3);
        merkle_sum_tree.push(leaf_4);
        merkle_sum_tree.push(leaf_5);
        merkle_sum_tree.remove(3);
        println!("{:?}", merkle_sum_tree.get_zero_index());
        println!("{:?}", merkle_sum_tree.get_leaf(3));
        println!("{:?}", merkle_sum_tree.get_leafs());
    }
}
