// Import the core library
extern crate ipfs_host;
extern crate ipfs_core;
use std::{str, time::Instant};
use std::env;
use std::sync::Arc;
use std::sync::Mutex;
use axum::response::Response;
use bincode::Error;
use ::bonsai_sdk::alpha::responses::SnarkReceipt;

use ethers::abi::{Token, Tokenizable};
use ethers::types::U256;
use ipfs_core::ProofReceipt;
use std::time::Duration;
use std::result::Result;
use bs58;
use hex;
use methods::{VERIFY_IPFS_CONTENT_ELF, VERIFY_IPFS_CONTENT_ID};
use risc0_zkvm::{
    Receipt, serde::{to_vec, from_slice}, PAGE_SIZE, compute_image_id

};


use bonsai_sdk::alpha as bonsai_sdk;
use tokio::runtime::Runtime;

use axum::{extract::Json,response::IntoResponse, routing::post, Router, http::StatusCode, debug_handler};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use hyper::Server;
fn load_env() {
    dotenv::dotenv().ok();
}
 //529215 - 529299 : 
    //1085148		00768944084			BETHAN SARAH COLLINGRIDGE	BETHAN SARAH	COLLINGRIDGE						true
    //45623854
    //22gb: QmWXcbXFkmbFPtQRDanLdKu4zafUVtPdKzWiyFA18sHRkt
    //220mb : Qmdro5YY2inaDSye3vgx7nuLtcNga45kbWB11tvmW1Qx74
   //let runtime = Runtime::new().unwrap();
    //runtime.block_on(test_stuff());
       // let result = ipfs_host::functions::select_from_ipfs_generate_guest_input(
    //     "QmWXcbXFkmbFPtQRDanLdKu4zafUVtPdKzWiyFA18sHRkt", 
    //     //1, 300
    //     (262158 * 2) - 1000, 
    //     (262158 * 2) + 1000,
        
    // ).await;
async fn test_stuff() {
    //ipfs_host::functions::get_block_bytes(hash)
    let result =ipfs_host::v0_proof::select_from_ipfs_generate_guest_input(
        "baguqeerasords4njcts6vs7qvdjfcvgnume4hqohf65zsfguprqphs3icwea", 
        //1, 300
        176, 
        193,
        
    );
}



#[derive(Debug, Deserialize)]
pub struct BonsaiRequest {
    hash: String,
    start: usize,
    end: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BonsaiResponse { 
    imageId: Token,
    journal: Token,
    postStateDigest: Token
}

#[tokio::main] 
async fn main() {
    load_env();
    let app = Router::new().route("/generateproof", post(generate_proof));
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3001);

    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let server = Server::bind(&addr)
        .http1_keepalive(true)
        .serve(app.into_make_service());
    println!("Listening on {}", addr);
    server.await.unwrap();
}

pub async fn generate_proof2(Json(req): Json<BonsaiRequest>) -> StatusCode {
    StatusCode::ACCEPTED
}

pub async fn generate_proof(Json(req): Json<BonsaiRequest>) -> Json<BonsaiResponse> {
    
    let result = ipfs_host::v0_proof::select_from_ipfs_generate_guest_input(
        &req.hash.clone(), 
        req.start.clone() as u64, 
        req.end.clone() as u64,
    ).await;
 
    let start_time = std::time::Instant::now();
    let input_vec = to_vec(&result).unwrap();
    println!("InputLength {}", input_vec.len());
   
    let result = tokio::task::spawn_blocking(move || run_bonsai(input_vec).unwrap());
    let (receipt, snark, image_id) = result.await.unwrap();
    println!("Processing time: {} ms", start_time.elapsed().as_millis());
    
    Json(BonsaiResponse { 
            imageId: Token::FixedBytes(hex::decode(image_id).unwrap()),
            journal: Token::Bytes(snark.journal),
            //seal: Token::Bytes(ethers::abi::encode(&[tokenize_snark_receipt(&snark.snark).unwrap()])),
            postStateDigest: Token::FixedBytes(snark.post_state_digest)
       })
}
fn run_stark2snark(session_id: String) -> Result<SnarkReceipt, Box<dyn std::error::Error>> {
    let client = bonsai_sdk::Client::from_env("0.21.0").unwrap();

    let snark_session = client.create_snark(session_id).unwrap();
    tracing::info!("Created snark session: {}", snark_session.uuid);
    loop {
        let res = snark_session.status(&client).unwrap();
        match res.status.as_str() {
            "RUNNING" => {
                println!("Current status: {} - continue polling...", res.status,);
                std::thread::sleep(Duration::from_secs(3));
                continue;
            }
            "SUCCEEDED" => {
                let snark_receipt = res.output;
                println!("Snark proof!: {snark_receipt:?}");
                return Ok(snark_receipt.unwrap());
                
            }
            _ => {
                panic!("Workflow exited: {} err: {}", res.status, res.error_msg.unwrap_or_default());
            }
        }
    }
}

fn run_bonsai(input_data: Vec<u32>) -> Result<(Receipt, SnarkReceipt, String), Box<dyn std::error::Error>> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION).unwrap();

    // create the memoryImg, upload it and return the imageId
    let img_id = hex::encode(compute_image_id(VERIFY_IPFS_CONTENT_ELF)?);
        
    let rrr = client.upload_img(&img_id, VERIFY_IPFS_CONTENT_ELF.to_vec())?;
    

    println!("ImageID {} ", img_id);

    // Prepare input data and upload it.
    //let input_data = to_vec(&input_data).unwrap();
    let input_data = bytemuck::cast_slice(&input_data).to_vec();
    let input_id = client.upload_input(input_data).unwrap();

    // Start a session running the prover
    let session = client.create_session(img_id.clone(), input_id, vec![]).unwrap();
    println!("Sessionid: {}", session.uuid);
    loop {
        let res = match session.status(&client) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error getting session status: {}", err);
                continue;
            }
        };

        if res.status == "RUNNING" {
            println!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(3));
            continue;
        }

        if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = match client.download(&receipt_url) {
                Ok(buf) => buf,
                Err(err) => {
                    eprintln!("Error downloading receipt: {}", err);
                    continue;
                }
            };

            let receipt: Receipt = match bincode::deserialize(&receipt_buf) {
                Ok(receipt) => receipt,
                Err(err) => {
                    eprintln!("Error deserializing receipt: {}", err);
                    continue;
                }
            };

            let rrr: ProofReceipt = match from_slice(&receipt.journal.bytes) {
                Ok(rrr) => rrr,
                Err(err) => {
                    eprintln!("Error processing proof receipt: {}", err);
                    continue;
                }
            };

            println!("IPFS Data {:#?}", String::from_utf8(rrr.clone().data));
            println!("IPFS Hash {}", bs58::encode(&rrr.hash).into_string());

            if let Err(err) = receipt.verify(VERIFY_IPFS_CONTENT_ID) {
                eprintln!("Receipt verification failed: {}", err);
                continue;
            }

            let sss = match run_stark2snark(session.uuid.clone()) {
                Ok(sss) => sss,
                Err(err) => {
                    eprintln!("Error running stark2snark: {}", err);
                    continue;
                }
            };

            return Ok((receipt, sss, img_id));
        } else {
            panic!("Workflow exited: {} - | err: {}", res.status, res.error_msg.unwrap_or_default());
        }

        break;
    }

    panic!("Nope!")
}



// pub fn tokenize_snark_receipt(proof: &Groth16Seal) -> anyhow::Result<Token> {
    
//     Ok(Token::FixedArray(vec![
//         Token::FixedArray(
//             proof
//                 .a
//                 .iter()
//                 .map(|elm| U256::from_big_endian(elm).into_token())
//                 .collect(),
//         ),
//         Token::FixedArray(vec![
//             Token::FixedArray(
//                 proof.b[0]
//                     .iter()
//                     .map(|elm| U256::from_big_endian(elm).into_token())
//                     .collect(),
//             ),
//             Token::FixedArray(
//                 proof.b[1]
//                     .iter()
//                     .map(|elm| U256::from_big_endian(elm).into_token())
//                     .collect(),
//             ),
//         ]),
//         Token::FixedArray(
//             proof
//                 .c
//                 .iter()
//                 .map(|elm| U256::from_big_endian(elm).into_token())
//                 .collect(),
//         ),
//     ]))
// }