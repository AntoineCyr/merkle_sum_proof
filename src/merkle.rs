use crate::mimc_sponge::{Fr, MimcSponge};
use ff::{self, *};
use ntest_timeout::timeout;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// TO ADD:
/// Add value, change or remove value
/// Get Merkle Path
/// Verify inclusion with and without merkle path
/// Test everything

/// The list of nodes includes all nodes of the tree
/// It is in the form [h1, h2, h3, h4, h12, h34, h1234], where h1234 would be the root and h1-4 is a leaf.
/// The number of leafs needs to be power of 2. The tree will be filled with 0 to keep the power of 2 intact.
#[derive(Debug, Clone)]
pub struct MerkleSumTree {
    //hash function?
    //vec of 0 leafs
    leafs: Vec<Leaf>,
    nodes: Vec<Node>,
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
        let nodes = Self::create_tree(leafs.clone());
        MerkleSumTree { leafs, nodes }
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

    fn create_tree(mut leafs: Vec<Leaf>) -> Vec<Node> {
        leafs = Self::fill_leafs(leafs);
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
        if nodes_to_hash.len() == 1 {
            nodes.push(nodes_to_hash[0].clone());
        }

        nodes
    }

    fn fill_leafs(mut leafs: Vec<Leaf>) -> Vec<Leaf> {
        let mut power = 1;
        while power < leafs.len() {
            power = power << 1;
        }
        let empty_leaf = Leaf::new("0".to_string(), 0);
        for _ in 0..power - leafs.len() {
            leafs.push(empty_leaf.clone())
        }
        leafs
    }

    fn build_parent(child_1: Node, child_2: Node) -> Node {
        let arr = vec![
            child_1.get_hash(),
            Fr::from_str_vartime(&child_1.get_value().to_string()).unwrap(),
            child_2.get_hash(),
            Fr::from_str_vartime(&child_2.get_value().to_string()).unwrap(),
        ];
        println!("arr: {:?}", arr);
        let k = Fr::from_str_vartime("0").unwrap();
        let ms = MimcSponge::default();
        let hash = ms.multi_hash(&arr, k, 1);
        let sum = child_1.get_value() + child_2.get_value();
        Node::new(hash[0], sum)
    }
}

//arr: [Fr(0x0000000000000000000000000000000000000000000000003309c891ce14a103), Fr(0x000000000000000000000000000000000000000000000000c0cd4e53cd09276f), Fr(0x000000000000000000000000000000000000000000000000000000000000000a), Fr(0x000000000000000000000000000000000000000000000000000000000000000b)]
//res:  Fr(0x217799013c3ae265f5eef3457f10d9431aab7d2c5e758459dd26c0e5acd1a222)
#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    //#[timeout(1000)]
    fn it_works() {
        let leaf_1 = Leaf::new("11672136".to_string(), 10);
        let leaf_2 = Leaf::new("10566265".to_string(), 11);
        println!("leaf_1: {:?}", leaf_1);
        println!("leaf_1: {:?}", leaf_2);
        let merkle_sum_tree = MerkleSumTree::new(vec![leaf_1, leaf_2]);
        let root_hash = merkle_sum_tree.get_root_hash().unwrap();

        let root_sum = merkle_sum_tree.get_root_sum().unwrap();

        println!("root_hash: {:?}, root_sum: {:?}", root_hash, root_sum);
        //println!("nodes: {:?}", merkle_sum_tree.get_nodes());
        //println!("leafs: {:?}", merkle_sum_tree.get_leafs());
    }
}
