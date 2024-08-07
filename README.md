# Merkle Sum Tree Library

This library implements a Merkle Sum Tree data structure using Rust. It provides functionalities to create a Merkle Sum Tree, add and remove leaf nodes, generate and verify inclusion proofs, and retrieve tree properties.

## Features

- Create a Merkle Sum Tree from a list of leaf nodes.
- Add leaf nodes to the tree.
- Remove leaf nodes from the tree.
- Generate an inclusion proof for a given leaf node.
- Verify an inclusion proof.
- Retrieve the root hash and sum of the tree.
- Retrieve nodes, leafs, and tree height.

## Modules and Structs

### Modules

- `constants`: Contains constants used throughout the library.
- `mimc_sponge`: Contains the MiMC sponge function implementation.

### Structs

#### MerkleSumTree

A struct representing the Merkle Sum Tree.

- **Fields:**
  - `leafs: Vec<Leaf>`: A vector of leaf nodes.
  - `nodes: Vec<Node>`: A vector of nodes.
  - `height: usize`: The height of the tree.
  - `zero_index: Vec<usize>`: A vector containing the indices of zero-value nodes.

- **Methods:**
  - `new(leafs: Vec<Leaf>) -> Result<MerkleSumTree>`: Creates a new Merkle Sum Tree from a list of leaf nodes.
  - `get_root_hash(&self) -> Option<Fr>`: Returns the root hash of the tree.
  - `get_root_sum(&self) -> Option<i32>`: Returns the root sum of the tree.
  - `get_root(&self) -> Option<Node>`: Returns the root node of the tree.
  - `get_nodes(&self) -> Vec<Node>`: Returns all the nodes of the tree.
  - `get_leafs(&self) -> Vec<Leaf>`: Returns all the non-zero leafs of the tree.
  - `get_zero_index(&self) -> Vec<usize>`: Returns the zero index vector.
  - `get_node(&self, index: usize) -> Option<Node>`: Returns a node at a specific index.
  - `get_leaf(&self, index: usize) -> Option<Leaf>`: Returns a leaf at a specific index.
  - `get_height(&self) -> usize`: Returns the height of the tree.
  - `get_proof(&self, index: usize) -> Result<Option<InclusionProof>>`: Generates an inclusion proof for a given leaf node.
  - `verify_proof(&self, proof: InclusionProof) -> Result<bool>`: Verifies an inclusion proof.
  - `push(&mut self, leaf: Leaf) -> Result<usize>`: Adds a new leaf node to the tree and returns its index.
  - `set_leaf(&mut self, leaf: Leaf, index: usize) -> Result<()>`: Modifies a current leaf node.
  - `remove(&mut self, index: usize) -> Result<()>`: Removes a leaf node from the tree.

#### Leaf

A struct representing a leaf node in the Merkle Sum Tree.

- **Fields:**
  - `id: String`: The identifier of the leaf.
  - `node: Node`: The node associated with the leaf.

- **Methods:**
  - `new(id: String, value: i32) -> Leaf`: Creates a new leaf node with the given id and value.
  - `get_id(&self) -> String`: Returns the id of the leaf.
  - `get_node(&self) -> Node`: Returns the node associated with the leaf.
  - `is_none(&self) -> bool`: Checks if the leaf is a zero-value leaf.

#### Node

A struct representing a node in the Merkle Sum Tree.

- **Fields:**
  - `hash: Fr`: The hash of the node.
  - `value: i32`: The value of the node.

- **Methods:**
  - `new(hash: Fr, value: i32) -> Node`: Creates a new node with the given hash and value.
  - `get_hash(&self) -> Fr`: Returns the hash of the node.
  - `get_value(&self) -> i32`: Returns the value of the node.
  - `is_equal(&self, node: Node) -> bool`: Checks if the node is equal to another node.

#### InclusionProof

A struct representing an inclusion proof in the Merkle Sum Tree.

- **Fields:**
  - `leaf: Leaf`: The leaf node being proved.
  - `path: Vec<Neighbor>`: The path of neighbor nodes for the proof.

- **Methods:**
  - `get_path(&self) -> Vec<Neighbor>`: Returns the path of neighbor nodes.
  - `get_leaf(&self) -> Leaf`: Returns the leaf node being proved.

#### Neighbor

A struct representing a neighbor node in the Merkle Sum Tree.

- **Fields:**
  - `position: Position`: The position of the neighbor node (Left or Right).
  - `node: Node`: The neighbor node.

- **Methods:**
  - `new(position: Position, node: Node) -> Neighbor`: Creates a new neighbor node with the given position and node.
  - `get_position(&self) -> Position`: Returns the position of the neighbor node.
  - `get_node(&self) -> Node`: Returns the neighbor node.

#### Position

An enum representing the position of a neighbor node in the Merkle Sum Tree.

- **Variants:**
  - `Left`: The neighbor node is on the left.
  - `Right`: The neighbor node is on the right.
