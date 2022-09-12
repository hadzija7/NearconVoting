use semaphore::{hash_to_field, Field, identity::Identity, poseidon_tree::PoseidonTree,
    protocol::* , merkle_tree::Branch};

use color_eyre::Result;

use serde_json::{json, to_writer};
use std::fs::File;

#[ignore]
#[test]
fn semaphore() -> Result<()> {
    // generate identity
    let id = Identity::from_seed(b"secret");
    
    // generate merkle tree
    let leaf = Field::from(0);
    let mut tree = PoseidonTree::new(4, leaf);
    tree.set(0, id.commitment());
    
    let merkle_proof = tree.proof(0).expect("proof should exist");
    let root = tree.root();
    
    // change signal and external_nullifier here
    let signal_hash = hash_to_field(b"xxx");
    let external_nullifier_hash = hash_to_field(b"appId");
    
    let nullifier_hash = generate_nullifier_hash(&id, external_nullifier_hash);

    let mut tree_path_indices= vec![];
    let mut tree_siblings = vec![];

    for elem in merkle_proof.0.iter() {        
        match elem {
            Branch::Left(c) => {
                tree_path_indices.push("0".to_string());
                tree_siblings.push(c.to_string());
            },
            Branch::Right(c) => {
                tree_path_indices.push("1".to_string());
                tree_siblings.push(c.to_string());
            },
        }
    }
    
    let input = json!({    
        "identityNullifier": id.nullifier.to_string(),
        "identityTrapdoor": id.trapdoor.to_string(),
        "treePathIndices": tree_path_indices,
        "treeSiblings": tree_siblings,
        "signalHash": signal_hash.to_string(),
        "externalNullifier": external_nullifier_hash.to_string(),
    });

    // println!("{}", input);
    
    to_writer(&File::create("circuits/input.json")?, &input)?;


    // println!("root: {}", root);
    
    let proof = generate_proof(&id, &merkle_proof, external_nullifier_hash, signal_hash).unwrap();
    let success = verify_proof(root, nullifier_hash, signal_hash, external_nullifier_hash, &proof).unwrap();
    
    assert!(success);

    Ok(())
}