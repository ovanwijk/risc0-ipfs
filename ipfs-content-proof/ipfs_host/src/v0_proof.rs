use async_recursion::async_recursion;
use bytes::Bytes;
use ipfs_api_backend_hyper::TryFromUri;
use ipfs_core::ProofType;
use sha2;
use http::uri::Scheme;
use bytes::BytesMut;
use futures::executor::block_on;
use futures::lock::Mutex;
use prost::Message;
use ipfs_core::IpfsProof;
use ipfs_api_backend_hyper::IpfsClient;
use ipfs_api_backend_hyper::IpfsApi;

use ipfs_messages::messages;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::vec;

use futures::TryStreamExt;
use std::io::{self, Write};
use hex::encode;
pub fn hello_world() {
    println!("Hello, world!");
}

/*
    A single link is 46 bytes or 45 or 44, as shrinking in containing size, max 6 bytes (280 terrabyte)
    Empty 'data' is 28 bytes
    Block size is varint, based on blocksize, max 6 bytes

*/

fn find_pattern_in_vec(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack.windows(needle.len())
        .position(|window| window == needle)
}

fn cut_vec(vec: Vec<u8>, index: usize, length: usize) -> (Vec<u8>, Vec<u8>) {
    let left = vec[..index].to_vec();
    let right = vec[index + length..].to_vec();
    (left, right)
}



pub struct SingleDataEntry {
    raw: Vec<Vec<u8>>,
    nodes: Vec<messages::PbNode>, 
    subset: Vec<u8>,

}

//Some default IPFS pre-fixes
pub const SHA256_PREFIX: [u8; 2] = [18, 32];
pub const DAG_PB_PREFIX: [u8; 4] = [1, 112, 18, 32];
pub const RAW_PREFIX: [u8; 4] = [1, 85, 18, 32];



pub fn build_proof(
    current_raw: Vec<u8>,
    current_node:messages::PbNode,
    subset:Vec<u8>,
    branch_map:HashMap<Vec<u8>, (Vec<u8>, messages::PbNode, Vec<u8>)>, 
    array_position:u64) -> (Vec<ProofType>, Vec<(u64, u64, u64)>, u64) {
        let mut position = array_position.clone();
        let mut left_over_bytes = current_raw.clone();
        let mut to_return: Vec<ProofType> = vec![];
        let mut selectors: Vec<(u64, u64, u64)> = vec![];
        //Maybe do a find in raw here ?
        if current_node.links.is_empty() && subset.len() > 0 {
                    
            let data_position_start = find_pattern_in_vec(current_raw.clone().as_slice(), subset.clone().as_slice()).unwrap();     
            selectors.push((position, data_position_start as u64, subset.len() as u64));
            println!("Adding data selection {}- {}", data_position_start, subset.clone().len());
            to_return.push(ProofType::Raw(left_over_bytes));
            position += 1;
            //Final position
        }else{
            for link in current_node.links {
                if branch_map.contains_key(&link.clone().hash.unwrap()) {
                    //Here we cut out the hash from the original raw bytes and are left with a left, right array.
                    //We add a ProofType::Branch in here. The hash from those raw bytes will need to be fit in
                    // left-right later during proof generation.
                    let data_position_start = find_pattern_in_vec(left_over_bytes.clone().as_slice(), link.clone().hash.unwrap().as_slice()).unwrap();
                    let data_cut_length = link.clone().hash.unwrap().len();
                    let (left, right) = cut_vec(left_over_bytes.clone(), data_position_start, data_cut_length);
                    to_return.push(ProofType::Raw(left));
                    position += 1;
                    let (raw, node, subset) = branch_map.get(&link.clone().hash.unwrap()).unwrap();
                    let (proofs
                        , new_selectors
                        , new_position) = build_proof(raw.clone(), node.clone(), subset.clone(), branch_map.clone(), position.clone() + 1);
                    selectors.extend(new_selectors);
                    to_return.push(ProofType::Branch(proofs));
                    position = new_position;
                    left_over_bytes = right;

                }
            }
            to_return.push(ProofType::Raw(left_over_bytes));
            position += 1;
            
        }
        
    (to_return, selectors, position)
}

use sha2::{Sha256, Digest};

pub async fn select_from_ipfs_generate_guest_input(hash: &str, start: u64, end: u64) -> IpfsProof {
    let (_, _, found_entries) = depth_first_search(hash, 0, start, end, vec![], vec![]).await;
    let mut hm:HashMap<Vec<u8>, (Vec<u8>, messages::PbNode,Vec<u8>)> = HashMap::new();
    let res = get_block_bytes(hash).await;
    println!("Data length: {}", end - start);
            
    //Create lookup table.
    for i in 0..found_entries.len() {
        println!("----");
        for n in 0..found_entries[i].nodes.len() {
                      
            let mut hasher = Sha256::new();
            hasher.update(found_entries[i].raw[n].clone());
            let mut hashed_result:Vec<u8> = Vec::new();
            hashed_result.extend_from_slice(&SHA256_PREFIX);
            hashed_result.extend(hasher.finalize().to_vec());
            hm.insert(hashed_result.clone(), (
                found_entries[i].raw[n].clone(),
                found_entries[i].nodes[n].clone(),
                found_entries[i].subset.clone()));
        }
    }
    let mut hasher = Sha256::new();
    hasher.update(&res);
    let mut hashed_result:Vec<u8> = Vec::new();
    hashed_result.extend_from_slice(&SHA256_PREFIX);
    hashed_result.extend(hasher.finalize().to_vec());
    let original_data = &hm.get(&hashed_result).unwrap().2;
    let pb_node = messages::PbNode::decode(&mut Cursor::new(&res)).unwrap();
    let (proof, selectors,_) = build_proof(res.clone(), pb_node, original_data.clone(), hm, 0);
    let mut result_map: HashMap<u64, (u64, u64)> = HashMap::new();
    for item in selectors.clone(){
        result_map.insert(item.0, (item.1, item.2));
    }
    let to_return = IpfsProof{
        proof: proof,
        data_selector: result_map
    };
    let ressss = to_return.calculate_proof();
    println!("Does it work? {}", bs58::encode(ressss.hash).into_string());
    to_return

}



pub async fn get_block_bytes(hash:&str) -> Vec<u8> {
    println!("Getting hash: {}", hash);
    println!("Getting hash: {}", std::env::var("IPFS_API").unwrap());
    let iphost = std::env::var("IPFS_API")
        .ok()        
        .unwrap_or("".to_string());
    let client = IpfsClient::from_multiaddr_str(&iphost).unwrap();
    let hash_clone = hash.clone().to_owned();
   
    let result = tokio::task::spawn_blocking(move || {
        block_on(client.block_get(&hash_clone)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat())
    }).await.expect("Not to crash");
    match result {
        Ok(bytes) => bytes,
        Err(_) => vec![], // handle error appropriately
    }
}

#[async_recursion]
pub async fn depth_first_search(hash: &str, current_data_position: u64, start: u64, end: u64, history: Vec<messages::PbNode>, raw_history:Vec<Vec<u8>>) -> (Vec<u8>, u64, Vec<SingleDataEntry>) {
    //TODO we need 2 positions, 1 for actual data extraction and 1 for tree search, treesearch should be 
    // measured in an offset to the start and end.
    println!("Executing {} {} ", hash, current_data_position);
    let res = get_block_bytes(hash).await;
    //println!("{}", hex::encode(res.clone()));
    let pb_node = messages::PbNode::decode(&mut Cursor::new(&res)).unwrap();
    
    let pn_node_clone = pb_node.clone();
    let pb_node_data = messages::Data::decode(&mut Cursor::new( pb_node.data.unwrap().clone())).unwrap();
    
    let mut sub_selection = Vec::new();
    let mut new_data_position = current_data_position;
   
    let mut return_set:Vec<SingleDataEntry> = Vec::new();
    let mut new_history = history.clone();
    let mut new_raw_history = raw_history.clone();
    new_history.push(pn_node_clone.clone());
    new_raw_history.push(res.clone());
    if pb_node.links.is_empty() {
        
        let select_max_length = end - start;
        let data_len = pb_node_data.data.clone().unwrap().len() as u64;
        
        let new_end = current_data_position + data_len;
        let data_in_full_range = start > current_data_position && end < new_end;  // ...[...{..}..]....
        let range_fully_in_data = start < current_data_position && end > new_end; // ..{.[.......].}...
        let data_started = start > current_data_position && start < new_end && end > new_end; // ...[..{.....]..}..
        let data_ended = start < current_data_position && end > current_data_position && end < new_end; // ..{.[......}.]....
         
        if data_in_full_range || range_fully_in_data ||  data_started || data_ended {
            
            let start_cut = if start > current_data_position { start - current_data_position  - 1} else { 0 };
            //let end_cut = std::cmp::min(std::cmp::min(data_len - 1, start_cut + select_max_length), end + start - current_data_position) ; //end
            let end_cut = std::cmp::min(data_len - 1, start_cut + (select_max_length + start - current_data_position)) ; //end
        
            //nodes.push(pn_node_clone.clone());
            sub_selection = pb_node_data.data.unwrap()[(start_cut) as usize..(end_cut) as usize].to_vec();
            println!("Sub selection{}", sub_selection.len());
            
            return_set.push(SingleDataEntry {
                raw: new_raw_history.clone(),
                nodes: new_history.clone(),
                subset: sub_selection.clone()
            });
        }
        new_data_position = new_end;
       
        (sub_selection, new_data_position, return_set)
    } else {
        
        for link in pb_node.links {
                if new_data_position < end {
                    let hash2 = &bs58::encode(&link.hash.unwrap()).into_string();
                        
                        let (new_sub_selection, data_position, result_vecs) = 
                        depth_first_search( 
                                &hash2,
                                new_data_position.clone(), start, end, new_history.clone(), new_raw_history.clone()).await;
                        return_set.extend(result_vecs);
                        
                        new_data_position = data_position.clone();
                }
                
            }
            //println!("Curret size:{}, start: {} - ", current_size, start);
            (sub_selection, new_data_position, return_set)
        }
    

    
}
