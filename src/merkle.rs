use crate::mimc_sponge::{Fr, MimcSponge};
use ff::{self, *};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// TO ADD:
/// Specific get_leaf with index, get node with index
/// Add value, change or remove value
/// Get Merkle Path
/// Verify inclusion with and without merkle path
/// Handle empty tree
/// Test everything

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
}

#[derive(Debug, Clone)]
//change string for more specific data type
pub struct Leaf {
    id: String,
    id_hash: Fr,
    value: i32,
}

#[derive(Debug, Clone)]
pub struct Node {
    hash: Fr,
    value: i32,
}

#[derive(Debug, Clone)]
pub struct InclusionProof {
    index: usize,
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
}

impl Leaf {
    pub fn new(id: String, value: i32) -> Leaf {
        let mut hr = DefaultHasher::new();
        id.hash(&mut hr);
        let id_hash = Fr::from_u128(hr.finish() as u128);
        Leaf { id, id_hash, value }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_id_hash(&self) -> Fr {
        self.id_hash.clone()
    }
    pub fn get_value(&self) -> i32 {
        self.value.clone()
    }
}

impl MerkleSumTree {
    pub fn new(leafs: Vec<Leaf>) -> MerkleSumTree {
        println!("new tree");
        let (nodes, height) = Self::create_tree(leafs.clone());
        MerkleSumTree {
            leafs,
            nodes,
            height,
        }
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

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }

    pub fn get_leafs(&self) -> Vec<Leaf> {
        self.leafs.clone()
    }

    pub fn get_height(&self) -> usize {
        self.height.clone()
    }

    pub fn get_proof(&self, index: usize) -> Option<InclusionProof> {
        if self.get_leafs().len() < index {
            return None;
        }
        let leaf = self.get_leafs()[index].clone();
        let mut path = vec![];
        let height = self.get_height();
        let mut level_size = 1 >> height;
        let mut level_index = index;
        let mut current_index = index;
        for _ in 1..height {
            if current_index % 2 == 0 {
                let node = self.get_nodes()[index + 1].clone();
                let neighbor = Neighbor {
                    position: Position::Right,
                    node,
                };
                path.push(neighbor);
            } else {
                let node = self.get_nodes()[index - 1].clone();
                let neighbor = Neighbor {
                    position: Position::Left,
                    node,
                };
                path.push(neighbor);
            }
            current_index = current_index - level_index / 2 + level_size;
            level_index = level_index / 2;
            level_size = level_size / 2;
        }
        Some(InclusionProof { index, leaf, path })
    }

    fn create_tree(mut leafs: Vec<Leaf>) -> (Vec<Node>, usize) {
        let mut height;
        (leafs, height) = Self::fill_leafs(leafs);
        let mut nodes: Vec<Node> = vec![];
        let mut nodes_to_hash: Vec<Node> = vec![];
        let mut temp_hash_nodes: Vec<Node> = vec![];
        for leaf in leafs.iter() {
            let node = Node::new(leaf.get_id_hash(), leaf.get_value());
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
        (nodes, height)
    }

    fn fill_leafs(mut leafs: Vec<Leaf>) -> (Vec<Leaf>, usize) {
        let mut power = 1;
        let mut height = 1;
        while power < leafs.len() {
            power = power << 1;
            height += 1
        }
        let empty_leaf = Leaf::new("0".to_string(), 0);
        for _ in 0..power - leafs.len() {
            leafs.push(empty_leaf.clone())
        }
        (leafs, height)
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
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn it_works() {
        let leaf_1 = Leaf::new("11672136".to_string(), 10);
        let leaf_2 = Leaf::new("10566265".to_string(), 11);
        let leaf_3 = Leaf::new("10566215".to_string(), 12);

        let merkle_sum_tree = MerkleSumTree::new(vec![leaf_1, leaf_2, leaf_3]);
        let root_hash = merkle_sum_tree.get_root_hash().unwrap();
        let root_sum = merkle_sum_tree.get_root_sum().unwrap();
        let height = merkle_sum_tree.get_height();
        let proof = merkle_sum_tree.get_proof(0).unwrap();

        let nodes = merkle_sum_tree.get_nodes();
        //println!("root_hash: {:?}, root_sum: {:?}", root_hash, root_sum);
        //println!("nodes: {:?}", nodes);
        //println!("height: {:?}", height);
        println!("path: {:?}", proof.path);
    }
}
