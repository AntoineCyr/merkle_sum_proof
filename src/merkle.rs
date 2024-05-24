use crate::mimc_sponge::{Fr, MimcSponge};
use ff::{self, *};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// TO ADD:
/// Handle empty tree
/// Decide if should use Result or Option
/// Option is used for presence of value, and Result for error handling
/// Do Rust tutorial, make some change to make this better

/// The list of nodes includes all nodes of the tree
/// It is in the form [h1, h2, h3, h4, h12, h34, h1234], where h1234 would be the root and h1-4 is a leaf.
/// The number of leafs needs to be power of 2. The tree will be filled with 0 to keep the power of 2 intact.
/// Leafs will return only the leafs without the 0 values.
/// Nodes will return all nodes inlcuding the 0 value leafs.
#[derive(Debug, Clone)]
pub struct MerkleSumTree {
    //hash function?
    //vec of 0 leafs
    leafs: Vec<Leaf>,
    nodes: Vec<Node>,
    height: usize,
    zero_index: Vec<usize>,
}

#[derive(Debug, Clone)]
//change string for more specific data type
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
    pub fn new(leafs: Vec<Leaf>) -> MerkleSumTree {
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
            n => Some(self.get_node(n - 1)),
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

    pub fn get_node(&self, index: usize) -> Node {
        self.nodes[index].clone()
    }

    pub fn get_leaf(&self, index: usize) -> Leaf {
        self.leafs[index].clone()
    }

    pub fn get_height(&self) -> usize {
        self.height.clone()
    }

    fn update_path(&mut self, leaf: Leaf, index: usize) {
        let height = self.height;
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        let mut current_node = leaf.get_node();
        for _ in 1..height {
            if current_index % 2 == 0 {
                let neighbor = self.get_node(current_index + 1);
                current_node = Self::build_parent(current_node, neighbor);
            } else {
                let neighbor = self.get_node(current_index - 1);
                current_node = Self::build_parent(neighbor, current_node);
            }
            level_start += level_size;
            level_index = level_index / 2;
            current_index = level_start + level_index;
            level_size = level_size / 2;
            self.nodes[current_index] = current_node.clone();
            println!("current_index: {}", current_index);
            println!("current_node value: {}", current_node.get_value());
        }
    }

    pub fn get_proof(&self, index: usize) -> Option<InclusionProof> {
        if self.get_leafs().len() <= index {
            return None;
        }
        let leaf = self.get_leaf(index);
        let mut path = vec![];
        let height = self.get_height();
        let mut level_size = 1 << (height - 1);
        let mut level_index = index;
        let mut current_index = index;
        let mut level_start = 0;
        for _ in 1..height {
            if current_index % 2 == 0 {
                let node = self.get_node(current_index + 1);
                let neighbor = Neighbor {
                    position: Position::Right,
                    node,
                };
                path.push(neighbor);
            } else {
                let node = self.get_node(current_index - 1);
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
        Some(InclusionProof { leaf, path })
    }

    //maybe verify proof without the tree, just root hash
    //create a function to convert leaf to node
    pub fn verify_proof(&self, proof: InclusionProof) -> bool {
        let mut node = proof.leaf.get_node();
        let path = proof.path;

        for neighbor in path {
            match neighbor.position {
                Position::Right => {
                    node = Self::build_parent(node, neighbor.node);
                }
                Position::Left => {
                    node = Self::build_parent(neighbor.node, node);
                }
            }
        }
        node.is_equal(self.get_root().unwrap())
    }

    fn create_tree(mut leafs: Vec<Leaf>) -> MerkleSumTree {
        let height;
        let mut zero_index = vec![];
        (leafs, height, zero_index) = Self::fill_leafs(leafs);
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
                    Self::build_parent(nodes_to_hash[j].clone(), nodes_to_hash[j + 1].clone());
                nodes.push(new_node.clone());
                temp_hash_nodes.push(new_node);
                j += 2;
            }
            nodes_to_hash = temp_hash_nodes.clone();
            temp_hash_nodes = vec![];
        }
        MerkleSumTree {
            leafs,
            nodes,
            height,
            zero_index,
        }
    }

    fn fill_leafs(mut leafs: Vec<Leaf>) -> (Vec<Leaf>, usize, Vec<usize>) {
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
        (leafs, height, zero_index)
    }

    fn build_parent(child_1: Node, child_2: Node) -> Node {
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
        Node::new(hash[0], sum)
    }

    //Push new leaf, return index
    pub fn push(&mut self, leaf: Leaf) -> usize {
        match self.zero_index.len() {
            0 => {
                self.leafs.push(leaf);
                let index_value = self.leafs.len();
                let new_tree = Self::create_tree(self.leafs.clone());
                self.update_tree(new_tree);
                index_value
            }
            _ => {
                let index_value = self.zero_index[0];
                self.set_leaf(leaf, index_value);
                index_value
            }
        }
    }
    //check for empty leaf
    pub fn set_leaf(&mut self, leaf: Leaf, index: usize) {
        if leaf.is_none() && !self.get_leaf(index).is_none() {
            let pos = self.zero_index.binary_search(&index).unwrap_or_else(|e| e);
            self.zero_index.insert(pos, index);
        }
        if !leaf.is_none() && self.get_leaf(index).is_none() {
            let pos = self.zero_index.binary_search(&index).unwrap_or_else(|e| e);
            self.zero_index.remove(pos);
        }
        self.leafs[index] = leaf.clone();
        self.nodes[index] = leaf.get_node();
        self.update_path(leaf, index);
    }

    pub fn remove(&mut self, index: usize) {
        let leaf = Leaf::new("0".to_string(), 0);
        self.set_leaf(leaf, index);
    }

    fn update_tree(&mut self, tree: MerkleSumTree) {
        self.leafs = tree.leafs;
        self.nodes = tree.nodes;
        self.height = tree.height;
        self.zero_index = tree.zero_index;
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

        let mut merkle_sum_tree = MerkleSumTree::new(vec![leaf_1.clone(), leaf_2, leaf_3, leaf_4]);
        //merkle_sum_tree.push(leaf_1);
        //root_hash: Fr(0x2d2772b8cb7f2484bad633e9c882a42dc72b4bc9dc407f9657bddfc2062f7ff2), root_sum: 60
        let root_hash = merkle_sum_tree.get_root_hash().unwrap();
        let root_sum = merkle_sum_tree.get_root_sum().unwrap();
        let height = merkle_sum_tree.get_height();
        let proof = merkle_sum_tree.get_proof(1).unwrap();

        let nodes = merkle_sum_tree.get_nodes();
        let zero_index = merkle_sum_tree.get_zero_index();
        let included = merkle_sum_tree.verify_proof(proof.clone());
        println!("root_hash: {:?}, root_sum: {:?}", root_hash, root_sum);
        //println!("nodes: {:?}", nodes);
        //println!("height: {:?}", height);
        //println!("path: {:?}", proof.path);
        println!("included: {:?}", merkle_sum_tree.verify_proof(proof));
        println!("zero_index: {:?}", zero_index)
    }
}
