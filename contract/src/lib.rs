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
use near_sdk::{log, near_bindgen};

use semaphore_custom::{
    hash_to_field, identity::Identity, merkle_tree::Branch, poseidon_tree::PoseidonTree, Field,
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
   "8879890989264990724929583572227694093020547395165234611490038863301440480124",
   "9118915921200321700674476121032174636824825361092912880111756114618259932218"
  ],
  [
   "4656010635710846748280498970499223214928145030500196139272611309304666324119",
   "20659316531056419097669569166696372825924058522157165616319944674492475650003"
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
   "7992849163359690277429275172137171952092326291810396064293381898445871519950",
   "17546929511291163694211970903337129823288895193302653942313669095348015551545",
   "1"
  ],
  [
   "20455173372438527448587104721675651359020650404150064947152976639148022496703",
   "11202675299758796486930085599153684972945186546399818930452942776397882101575",
   "1"
  ],
  [
   "16152531242406568656841758663925122383265787230676884611802939204195545521540",
   "1911617130081794819725040297845710381910455918022984655713792331988504663797",
   "1"
  ],
  [
   "14192820197289767492279902003053886295813080519670340955603008735291114325607",
   "20808565309247786727521680832632637973834830526153037852919157047604245846002",
   "1"
  ],
  [
   "16268374589088317723741174014208052657921533591444943021723351576506766999010",
   "11272411066061777455868323777059996812926976899902173672950172003710349844986",
   "1"
  ]
 ]
}
"#;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    merkle_tree: MerkleTree,
    nullifiers: HashMap<String, HashSet<String>>, // external_nullifier: < nullifier_hash  >
    next_leaf: usize,
}

#[derive(PartialEq)]
pub struct MerkleTree {
    tree: PoseidonTree,
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self {
            tree: PoseidonTree::new(4, Field::from(0)),
        }
    }
}

impl BorshDeserialize for MerkleTree {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let depth = usize::deserialize(buf)?;

        let empty = Vec::<String>::deserialize(buf)?.iter().map(|x| x.parse::<Field>().unwrap()).collect();

        let nodes = Vec::<String>::deserialize(buf)?.iter().map(|x| x.parse::<Field>().unwrap()).collect();

        let merkle_tree = MerkleTree{
            tree: PoseidonTree {
                depth: depth,
                empty: empty,
                nodes: nodes,
            }
        };

        Ok(merkle_tree)
    }
}

impl BorshSerialize for MerkleTree {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        
        self.tree.depth.serialize(writer)?;
        self.tree
            .empty
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .serialize(writer)?;
        self.tree
            .nodes
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .serialize(writer)?;

        Ok(())
    }
}

#[derive(Debug, near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
pub struct MerklePath {
    tree_path_indices: Vec<String>,
    tree_siblings: Vec<String>,
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
        self.merkle_tree
            .tree
            .set(self.next_leaf, commitment.parse::<Field>().unwrap());
        self.next_leaf += 1;
    }

    pub fn get_root(&self) -> String {
        self.merkle_tree.tree.root().to_string()
    }

    pub fn get_next_leaf(&self) -> usize {
        self.next_leaf
    }

    pub fn add_poll(&mut self, poll_id: String) {
        // TODO: add Poll struct
        // let poll_id = poll_id_str.parse::<Field>().unwrap();

        assert!(
            !self.nullifiers.contains_key(&poll_id),
            "poll already exists"
        );

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

        assert!(
            self.nullifiers.contains_key(&pub_inputs[3].to_string()),
            "poll doesn't exist"
        );

        assert!(
            !self
                .nullifiers
                .get(&pub_inputs[3].to_string())
                .unwrap()
                .contains(&pub_inputs[1].to_string()),
            "used nullifier"
        );

        assert!(pub_inputs[0] == self.merkle_tree.tree.root(), "wrong root");

        assert!(
            pub_inputs[2] == hash_to_field(signal.as_bytes()),
            "mismatched signal"
        );

        self.nullifiers
            .entry(pub_inputs[3].to_string())
            .and_modify(|set| {
                set.insert(pub_inputs[1].to_string());
            });

        log!("Verified and emitting signal: {}", signal);
    }

    pub fn get_branch(&self, leaf_index: usize) -> MerklePath {
        let merkle_proof = self
            .merkle_tree
            .tree
            .proof(leaf_index)
            .expect("proof should exist");

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
            tree_path_indices: tree_path_indices,
            tree_siblings: tree_siblings,
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
    use near_sdk::serde::{Deserialize, Serialize};
    use serde_json::from_reader;
    use std::fs::{read_to_string, File};
    use std::io::{BufReader, BufRead};

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

        #[derive(Debug, Serialize, Deserialize)]
        struct Json {
            treePathIndices: Vec<String>,
            treeSiblings: Vec<String>,
        }

        // Read the JSON contents of the file as an instance of `User`.
        let input: Json = from_reader(reader).unwrap();

        let merkle_path = contract.get_branch(0);

        assert_eq!(input.treePathIndices, merkle_path.tree_path_indices);
        assert_eq!(input.treeSiblings, merkle_path.tree_siblings);
    }

    #[test]
    fn serialize_deserialize() {
        let tree = MerkleTree::default();

        let mut buf = File::create("tree.buf").unwrap();

        tree.serialize(&mut buf).ok();

        let file = File::open("tree.buf").unwrap();
        let mut reader = BufReader::new(file);
        let mut file_buf = reader.fill_buf().unwrap();
        
        let buf_tree = MerkleTree::deserialize(&mut file_buf).unwrap();

        assert_eq!(tree.tree.depth, buf_tree.tree.depth);
        assert_eq!(tree.tree.empty, buf_tree.tree.empty);
        assert_eq!(tree.tree.nodes, buf_tree.tree.nodes);
    }
}
