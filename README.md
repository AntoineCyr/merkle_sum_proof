# Merkle Sum Tree

A Rust implementation of a Merkle Sum Tree using the MiMC hash function for zero-knowledge proof applications.

## Features

- **Tree Operations**: Create, add, remove, and update leaf nodes
- **Proof Generation**: Generate and verify inclusion proofs
- **Automatic Padding**: Tree size automatically adjusted to power of 2
- **Overflow Protection**: Safe integer arithmetic with overflow checking

## Core Types

### `MerkleSumTree`
Main tree structure with methods for:
- `new(leafs: Vec<Leaf>)` - Create tree from leaves
- `push(&mut self, leaf: Leaf)` - Add new leaf
- `remove(&mut self, index: usize)` - Remove leaf
- `get_proof(&self, index: usize)` - Generate inclusion proof
- `verify_proof(&self, proof: &InclusionProof)` - Verify proof
- `get_root_hash()`, `get_root_sum()` - Access root values

### `Leaf`
Represents a leaf node with an ID and value.
- `new(id: String, value: i32)` - Create leaf

### `Node`
Internal node containing hash and sum value.

### `InclusionProof`
Merkle proof for leaf membership verification.

## Constraints

- Maximum height: 64 levels
- Leaf values: `i32` integers with overflow protection
- Tree size: Must be power of 2 (auto-padded with zero-value leaves)
- Empty trees not allowed

## Design Rationale

### Custom Implementation
This repository implements its own Merkle Sum Tree rather than using existing libraries to maintain full control over the cryptographic primitives and tree structure. This enables:
- Fine-tuned optimization for zero-knowledge proof systems
- Direct integration with specific ZK frameworks
- Customizable hashing and field arithmetic

### Flexible Prime Field
Custom implementation with configurable prime field modulus. Currently uses Nova's field but can switch to BN254 (Circom) via recompilation. This enables compatibility with different ZK proof systems.

### Possible Extensions
- Runtime-configurable field and hash function selection
- Sparse tree variant
- Additional hash functions (Poseidon)