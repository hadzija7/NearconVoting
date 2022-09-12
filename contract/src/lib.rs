/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use electron_rs::verifier::near::{
    get_prepared_verifying_key, parse_verification_key, verify_proof,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{log, near_bindgen};

use semaphore_custom::{
    hash_to_field, identity::Identity, merkle_tree:: Branch, poseidon_tree::PoseidonTree, Field,
};
use std::collections::{HashMap, HashSet};

const VKEY_STR: &str = r#"
{
 "protocol": "groth16",
 "curve": "bn128",
 "nPublic": 4,
 "vk_alpha_1": [
  "20491192805390485299153009773594534940189261866228447918068658471970481763042",
  "9383485363053290200918347156157836566562967994039712273449902621266178545958",
  "1"
 ],
 "vk_beta_2": [
  [
   "6375614351688725206403948262868962793625744043794305715222011528459656738731",
   "4252822878758300859123897981450591353533073413197771768651442665752259397132"
  ],
  [
   "10505242626370262277552901082094356697409835680220590971873171140371331206856",
   "21847035105528745403288232691147584728191162732299865338377159692350059136679"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_gamma_2": [
  [
   "10857046999023057135944570762232829481370756359578518086990519993285655852781",
   "11559732032986387107991004021392285783925812861821192530917403151452391805634"
  ],
  [
   "8495653923123431417604973247489272438418190587263600148770280649306958101930",
   "4082367875863433681332203403145435568316851327593401208105741076214120093531"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_delta_2": [
  [
   "3821443919983577727088651801428802207724056406272428799071076005098787234763",
   "8174750903966358777454739020857516908911833594606212897386558063951586529110"
  ],
  [
   "18292198139401600943932694434056405837635210778597205508756160444400816932828",
   "21193267598291928284543677384385002232279126708130549822530350448367284765441"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_alphabeta_12": [
  [
   [
    "2029413683389138792403550203267699914886160938906632433982220835551125967885",
    "21072700047562757817161031222997517981543347628379360635925549008442030252106"
   ],
   [
    "5940354580057074848093997050200682056184807770593307860589430076672439820312",
    "12156638873931618554171829126792193045421052652279363021382169897324752428276"
   ],
   [
    "7898200236362823042373859371574133993780991612861777490112507062703164551277",
    "7074218545237549455313236346927434013100842096812539264420499035217050630853"
   ]
  ],
  [
   [
    "7077479683546002997211712695946002074877511277312570035766170199895071832130",
    "10093483419865920389913245021038182291233451549023025229112148274109565435465"
   ],
   [
    "4595479056700221319381530156280926371456704509942304414423590385166031118820",
    "19831328484489333784475432780421641293929726139240675179672856274388269393268"
   ],
   [
    "11934129596455521040620786944827826205713621633706285934057045369193958244500",
    "8037395052364110730298837004334506829870972346962140206007064471173334027475"
   ]
  ]
 ],
 "IC": [
  [
   "18909076149018184150507373559728243658390925434549795413114925967127955290677",
   "14103112161910936758168576360044522814990537316587849666682250230829746592099",
   "1"
  ],
  [
   "7969327040973863889335756470214138105748057763189479952697331459344138800764",
   "5785196400466483463132253298583166990390575866403759459623299335117125845465",
   "1"
  ],
  [
   "21051097487881164874463033375052129047394843744573294847111308392028731377030",
   "10855533610461666477249153082306690667863505352812708181028218370521655058492",
   "1"
  ],
  [
   "4319780315499060392574138782191013129592543766464046592208884866569377437627",
   "13920930439395002698339449999482247728129484070642079851312682993555105218086",
   "1"
  ],
  [
   "3554830803181375418665292545416227334138838284686406179598687755626325482686",
   "5951609174746846070367113593675211691311013364421437923470787371738135276998",
   "1"
  ]
 ]
}
"#;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    #[borsh_skip]
    merkle_tree: MerkleTree,
    nullifiers: HashMap<String, HashSet<String>>, // external_nullifier: < nullifier_hash  >
    next_leaf: usize,
}

pub struct MerkleTree {
    tree: PoseidonTree,
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self {
            tree: PoseidonTree::new(21, Field::from(0))
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct MerklePath {
    treePathIndices: Vec<String>,
    treeSiblings: Vec<String>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            merkle_tree: MerkleTree::default(),
            nullifiers: HashMap::new(),
            next_leaf: 0,
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    pub fn verify_proof_on_chain(&self, proof: String, inputs: String) -> bool {
        let vkey = parse_verification_key(VKEY_STR.to_string()).unwrap();
        let prepared_vkey = get_prepared_verifying_key(vkey);

        verify_proof(prepared_vkey, proof, inputs).unwrap()
    }

    pub fn insert_leaf(&mut self, commitment: String) {
        self.merkle_tree.tree.set(self.next_leaf, commitment.parse::<Field>().unwrap());
        self.next_leaf += 1;
    }

    pub fn get_root(&self) -> String {
        self.merkle_tree.tree.root().to_string()
    }

    pub fn get_next_leaf(&self) -> usize {
        self.next_leaf
    }

    pub fn add_poll(&mut self, poll_id: String) { // TODO: add Poll struct
        // let poll_id = poll_id_str.parse::<Field>().unwrap();

        assert!(!self.nullifiers.contains_key(&poll_id), "poll already exists");

        log!("Poll created: {}", &poll_id);

        self.nullifiers.insert(poll_id, HashSet::new());
    }

    pub fn vote(&mut self, signal: String, proof: String, inputs: String) {
        assert!(
            self.verify_proof_on_chain(proof, inputs.clone()),
            "invalid proof"
        );

        let pub_inputs: Vec<Field> = inputs
            .split("\"") // split string into words by whitespace
            .filter_map(|w| w.parse::<Field>().ok()) // calling ok() turns Result to Option so that filter_map can discard None values
            .collect();

        assert!(self.nullifiers.contains_key(&pub_inputs[3].to_string()), "poll doesn't exist");

        assert!(
            !self.nullifiers.get(&pub_inputs[3].to_string()).unwrap().contains(&pub_inputs[1].to_string()),
            "used nullifier"
        );

        assert!(pub_inputs[0] == self.merkle_tree.tree.root(), "wrong root");

        assert!(
            pub_inputs[2] == hash_to_field(signal.as_bytes()),
            "mismatched signal"
        );

        self.nullifiers.entry(pub_inputs[3].to_string()).and_modify(|set| {set.insert(pub_inputs[1].to_string());});

        log!("Verified and emitting signal: {}", signal);
    }

    pub fn get_branch(&self, leaf_index: usize) -> MerklePath {
        let merkle_proof = self.merkle_tree.tree.proof(leaf_index).expect("proof should exist");

        let mut tree_path_indices = vec![];
        let mut tree_siblings = vec![];

        for elem in merkle_proof.0.iter() {
            match elem {
                Branch::Left(c) => {
                    tree_path_indices.push("0".to_string());
                    tree_siblings.push(c.to_string());
                }
                Branch::Right(c) => {
                    tree_path_indices.push("1".to_string());
                    tree_siblings.push(c.to_string());
                }
            }
        }

        MerklePath {
            treePathIndices: tree_path_indices,
            treeSiblings: tree_siblings,
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_reader;
    use std::fs::{read_to_string, File};
    use std::io::BufReader;

    #[test]
    fn insert_new_leaf() {
        let mut contract = Contract::default();
        let original_root = contract.get_root();

        let id = Identity::from_seed(b"secret");

        contract.insert_leaf(id.commitment().to_string());

        let new_root = contract.get_root();
        let next_leaf = contract.get_next_leaf();

        assert_ne!(original_root, new_root);
        assert_eq!(next_leaf, 1);
    }

    #[test]
    fn add_poll() {
        let mut contract = Contract::default();

        let external_nullifier_hash = hash_to_field(b"appId");

        contract.add_poll(external_nullifier_hash.to_string());
    }

    #[test]
    #[should_panic(expected = "poll already exists")]
    fn cannot_add_poll_twice() {
        let mut contract = Contract::default();

        let external_nullifier_hash = hash_to_field(b"appId");

        contract.add_poll(external_nullifier_hash.to_string());
        contract.add_poll(external_nullifier_hash.to_string());
    }

    #[test]
    fn insert_new_leaf_and_vote() {
        let mut contract = Contract::default();

        let external_nullifier_hash = hash_to_field(b"appId");

        contract.add_poll(external_nullifier_hash.to_string());

        let id = Identity::from_seed(b"secret");

        contract.insert_leaf(id.commitment().to_string());

        let proof_str = read_to_string("circuits/proof.json").unwrap();

        let pub_input_str = read_to_string("circuits/public.json").unwrap();

        contract.vote("xxx".to_string(), proof_str, pub_input_str);
    }

    #[test]
    #[should_panic(expected = "used nullifier")]
    fn insert_new_leaf_and_vote_twice() {
        let mut contract = Contract::default();

        let external_nullifier_hash = hash_to_field(b"appId");

        contract.add_poll(external_nullifier_hash.to_string());

        let id = Identity::from_seed(b"secret");

        contract.insert_leaf(id.commitment().to_string());

        let proof_str = read_to_string("circuits/proof.json").unwrap();

        let pub_input_str = read_to_string("circuits/public.json").unwrap();

        contract.vote("xxx".to_string(), proof_str.clone(), pub_input_str.clone());
        contract.vote("xxx".to_string(), proof_str, pub_input_str);
    }

    #[test]
    #[should_panic(expected = "invalid proof")]
    fn insert_new_leaf_and_invalid_proof() {
        let mut contract = Contract::default();

        let id = Identity::from_seed(b"secret");

        contract.insert_leaf(id.commitment().to_string());

        let proof_str = read_to_string("circuits/proof.json").unwrap();

        let pub_input_str = r#"
        [
            "0","0","0","0"
        ]
        "#;

        contract.vote("xxx".to_string(), proof_str, pub_input_str.to_string());
    }

    #[test]
    fn insert_new_leaf_and_get_branch() {
        let mut contract = Contract::default();

        let id = Identity::from_seed(b"secret");

        contract.insert_leaf(id.commitment().to_string());
        // Open the file in read-only mode with buffer.
        let file = File::open("circuits/input.json").unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let input: MerklePath = from_reader(reader).unwrap();

        let merkle_path = contract.get_branch(0);

        assert_eq!(input, merkle_path);
    }
}
