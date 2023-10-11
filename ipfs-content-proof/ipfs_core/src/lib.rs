use std::collections::HashMap;
//use sha2;
use serde::{Deserialize, Serialize};
pub mod ipfsmessages;
//use risc0_zkvm::sha::rust_crypto::{Digest as _, Sha256} ;
use sha2::{Sha256, Digest};
pub const SHA256_PREFIX: [u8; 2] = [18, 32];
#[derive(Serialize, Deserialize, Clone)]
pub enum ProofType {
    Raw(Vec<u8>),
    Branch(Vec<ProofType>)
}
#[derive(Serialize, Deserialize)]
pub struct IpfsProof {
    pub proof: Vec<ProofType>,
    pub data_selector: HashMap<u64, (u64,u64)>
}

impl IpfsProof {
    // Return 0 is the hash, return 1 is the subselection of data
    pub fn calculate_proof(&self) -> (Vec<u8>, Vec<u8>) {
        
        
        let res = self.calculate_proof_req(&self.proof, vec![], 0);
        (res.0, res.1)
    }
    


    fn calculate_proof_req(&self, proofs:&Vec<ProofType>, res_data:Vec<u8>, index:u64) -> (Vec<u8>, Vec<u8>, u64) {
        
        let mut hasher = Sha256::new();
        let mut result_data = res_data;
        let mut new_index = index;
        for proof in proofs {
            match proof {
                ProofType::Raw(data) => {
                    hasher.update(data);
                    if let Some((start, end)) = self.data_selector.get(&(new_index as u64)) {
                        result_data.extend(&data[start.clone() as usize..(*start + *end) as usize]);

                    }
                    new_index += 1;
                },
                ProofType::Branch(branch) => {
                    let result = self.calculate_proof_req(branch,result_data.clone(), new_index + 1);
                    hasher.update(result.0);
                    result_data = result.1;
                    new_index = result.2;
                    
                },
            }
        }
        let mut hashed_result:Vec<u8> = Vec::new();
        hashed_result.extend_from_slice(&SHA256_PREFIX);
        hashed_result.extend(hasher.finalize().to_vec());
        println!("Exected index {}", new_index);
        (hashed_result, result_data, new_index)
    }
    
}