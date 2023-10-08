use async_recursion::async_recursion;
use bytes::Bytes;
use futures::executor::block_on;
use futures::lock::Mutex;
use ipfs_api_backend_actix::IpfsClient;
use ipfs_api_backend_actix::IpfsApi;
use ipfs_messages::messages;
use std::io::Cursor;
use std::sync::Arc;
use prost::Message;
use futures::TryStreamExt;
use std::io::{self, Write};
use hex::encode;
use hex_slice::AsHex;
pub fn hello_world() {
    println!("Hello, world!");
}

fn find_pattern_in_vec(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack.windows(needle.len())
        .position(|window| window == needle)
}

pub struct HashRequiredNode {
    parent: Option<Box<HashRequiredNode>>,
    parent_link_index: i32, //the index of the parent links, -1 for top node
    cid_start: u64, //Describes the start position in the parent raw bytes that must be equal
    cid_length: u64, //and the length of the CID,
    raw: Vec<u8>,
    node: messages::PbNode,
    data: messages::Data,
    must_resove: Vec<HashRequiredNode>

}

impl HashRequiredNode {
    pub fn new(parent: Option<Box<HashRequiredNode>>, parent_index:i32, raw:Vec<u8>, node: messages::PbNode,
        data: messages::Data, cid_start: u64,
        cid_length: u64) -> Self {
        Self {
            parent: parent,
            parent_link_index: parent_index,            
            raw: raw,
            cid_start: cid_start,
            cid_length:cid_length,
            node: node,
            data: data,
            must_resove: vec![]

        }
    }

    pub fn find_cid_range(&self, cid:&Vec<u8>) -> u64 {
        find_pattern_in_vec(&self.raw, cid).unwrap() as u64
    }
}
pub struct SingleDataEntry {
    nodes: Vec<messages::PbNode>, 
    datas: Vec<messages::Data> , 
    start: u64,
    end: u64
}




pub async fn get_block_bytes(hash:&str) -> Vec<u8> {
    println!("Getting hash: {}", hash);
    let client = IpfsClient::default();
    client.block_get(hash)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await.expect("Not to crash")
}
#[async_recursion(?Send)]
pub async fn depth_first_search(hash: &str, current_data_position: u64, start: u64, end: u64, history: Vec<messages::PbNode>) -> Result<(Vec<u8>, u64, Vec<SingleDataEntry>), Box<dyn std::error::Error>> {
    //TODO we need 2 positions, 1 for actual data extraction and 1 for tree search, treesearch should be 
    // measured in an offset to the start and end.
    println!("Executing {} {} ", hash, current_data_position);
    let mut res = get_block_bytes(hash).await;
    let mut pb_node = messages::PbNode::decode(&mut Cursor::new(&res))?;
    let pn_node_clone = pb_node.clone();
    let mut pb_node_data = messages::Data::decode(&mut Cursor::new( pb_node.data.unwrap().clone()))?;
    //let mut nodes = Vec::new();
    let mut sub_selection = Vec::new();
    let mut new_data_position = current_data_position;
    let mut return_set:Vec<SingleDataEntry> = Vec::new();
    let mut new_history = history.clone();
    new_history.push(pn_node_clone.clone());
    if pb_node.links.is_empty() {
        let select_max_length = end - start;
        let data_len = pb_node_data.data.clone().unwrap().len() as u64;
        
        let new_end = current_data_position + data_len;
        let data_in_full_range = start > current_data_position && end < new_end;  // ...[...{..}..]....
        let range_fully_in_data = start < current_data_position && end > new_end; // ..{.[.......].}...
        let data_started = start > current_data_position && start < new_end && end > new_end; // ...[..{.....]..}..
        let data_ended = start < current_data_position && end > current_data_position && end < new_end; // ..{.[......}.]....
        let data_before = current_data_position < start && new_end < start; // ...[.......]..{..}
        let data_after = current_data_position > end && new_end > end; // {..}.[.......]....
        //TODO add precise measuring
        if data_in_full_range || range_fully_in_data ||  data_started || data_ended {
            
            let start_cut = if start > current_data_position { start - current_data_position  - 1} else { 0 };
            let end_cut = std::cmp::min(data_len - 1, start_cut + select_max_length);
        
            //nodes.push(pn_node_clone.clone());
            sub_selection = pb_node_data.data.unwrap()[(start_cut) as usize..(end_cut) as usize].to_vec();
            let datas: Vec<messages::Data> = 
            new_history.iter().map(|node| {
                messages::Data::decode(&mut Cursor::new( node.clone().data.unwrap().clone())).unwrap()
                
            }).collect();
            return_set.push(SingleDataEntry {
                nodes: new_history.clone(),
                datas: datas,
                start: start_cut,
                end: end_cut
            });
        }
        new_data_position = new_end;
       
        Ok((sub_selection, new_data_position, return_set))
    } else {
        
        for link in pb_node.links {
                if new_data_position < end {
                    let hash2 = &bs58::encode(&link.hash.unwrap()).into_string();
                        
                        let (new_sub_selection, data_position, result_vecs) = 
                            depth_first_search( 
                                &hash2,
                                new_data_position.clone(), start, end, new_history.clone()).await?;
                        return_set.extend(result_vecs);
                        sub_selection.extend(new_sub_selection);
                        new_data_position = data_position.clone();
                }
                
            }
            //println!("Curret size:{}, start: {} - ", current_size, start);
            Ok((sub_selection, new_data_position, return_set))
        }
    

    
}




// pub async fn prepare_proof(path: Vec<String>, file: String, start: u64, end: u64) -> Result<PreparedIPFSProof, Box<dyn std::error::Error>> {
//     let client = IpfsClient::default();
  
    
//     let mut current_root = file.clone();

   
//     let mut res = get_block_bytes(&client, &current_root).await;

    

//     let mut pb_node = messages::PbNode::decode(&mut Cursor::new(&res))?;
//     let mut pb_node_data = messages::Data::decode(&mut Cursor::new( pb_node.data.unwrap()))?;

//     //Create toplinkNode
//     let mut proof = PreparedIPFSProof::new(start, end,
//          HashRequiredNode::new(None, -1, res.clone(), pb_node.clone(), pb_node_data.clone(), 0,0));
//     let mut current_node = proof.resolving_path;
//     let mut path_history = vec![(pb_node, pb_node_data)];

//     let mut file_byte_cursor = 0;
//     while file_byte_cursor < end {
//         //let (n,d) = current_node.node, current_node.data;
//         if messages::data::DataType::File as i32  != current_node.data.r#type.unwrap() {
//             return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Path must only contain files")));
//         }
        
        

//         //if not empty it means we have to go deeper
//         if !current_node.node.links.is_empty() {
//             proof.add_link_node(&current_node.node);
//             let mut counter = 0;
//             for blocksize in current_node.data.blocksizes {
//                 let new_end = file_byte_cursor + blocksize;
//                 let cid = &current_node.node.links[counter].hash.unwrap();
//                 // []= selector, {} = datarange
//                 let data_in_full_range = start > file_byte_cursor && end < new_end;  // ...[...{..}..]....
//                 let range_fully_in_data = start < file_byte_cursor && end > new_end; // ..{.[.......].}...
//                 let data_started = start > file_byte_cursor && end > new_end; // ...[..{.....]..}..
//                 let data_ended = start < file_byte_cursor && end < new_end; // ..{.[......}.]....
//                 let data_before = file_byte_cursor < start && new_end < start; // ...[.......]..{..}
//                 let data_after = file_byte_cursor > end && new_end > end; // {..}.[.......]....



//                 if data_started {
//                     res = get_block_bytes(&client, hex::encode(&cid).as_str()).await;
//                     pb_node = &messages::PbNode::decode(&mut Cursor::new(&res)).unwrap();
//                     pb_node_data = &messages::Data::decode(&mut Cursor::new( pb_node.data.unwrap())).unwrap();

                    
//                     if current_node.node.links.is_empty() {
//                         //This means it is a data-node, no walking down the tree anymore
//                         //Add to resolving nodes with data, then move back
//                         let new_node = HashRequiredNode::new(
//                             Some(current_node), 
//                             counter, 
//                             res.clone(), 
//                             pb_node.clone(), pb_node_data.clone(), current_node.find_cid_range(&cid), cid.len());
//                         current_node.must_resove.push(new_node);
//                     }else{

//                     }
                    
            
//                 }
//                 counter += 1;
//                 if file_byte_cursos + blocksize > start && 
//                 file_byte_cursos += blocksize;
                
//             }
//         }
        
//         res = get_block_bytes(&client, &current_root).await;

//         pb_node = messages::PbNode::decode(&mut Cursor::new(&res))?;
//         pb_node_data = messages::Data::decode(&mut Cursor::new( pb_node.data.unwrap()))?;

//     }
    
//     let file_cid = proof.get_last_node().links.iter().find(|link| link.name == file || link.hash == file).map(|link| link.hash).ok_or("File not found")?;
//     let res = client
//         .block_get(&file_cid)
//         .map_ok(|chunk| chunk.to_vec())
//         .try_concat()
//         .await?;

//     let pb_node = ipfs_messages::messages::PbNode::decode(&mut Cursor::new(&res))?;
//     proof.add_node(pb_node.clone());

//     let mut current_size = 0;
//     for blocksize in pb_node.data.blocksizes {
//         current_size += blocksize;
//         if current_size >= end {
//             let res = client
//                 .block_get(&pb_node.links.last().unwrap().hash)
//                 .map_ok(|chunk| chunk.to_vec())
//                 .try_concat()
//                 .await?;

//             let pb_node = ipfs_messages::messages::PbNode::decode(&mut Cursor::new(&res))?;
//             proof.add_node(pb_node);
//             break;
//         }
//     }

//     Ok(proof)
// }



    
// match client
// .block_get("QmWSFjkXStUiLpXK3bfS5Etk2CEQ9LN59uFjGnqmdbkAqM")
// .map_ok(|chunk| chunk.to_vec())
// .try_concat()
// .await
// {
// Ok(res) => {
//     let aaa = ipfs_messages::messages::PbNode::decode(&mut Cursor::new(&res));
//     let out = io::stdout();
//     let mut out = out.lock();
//     println!("{:}", encode(&res));
//     println!("{:x}", res.as_hex());
//     out.write_all(&res).unwrap();
// }
// Err(e) => eprintln!("error getting file: {}", e)
// }