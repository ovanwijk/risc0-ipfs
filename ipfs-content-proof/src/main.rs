// Import the core library
extern crate ipfs_host;
use std::str;
use bs58;
use methods::{VERIFY_IPFS_CONTENT_ELF, VERIFY_IPFS_CONTENT_ID};
use risc0_zkvm::{
    default_prover,
    serde::{from_slice, to_vec},    
    ExecutorEnv, Receipt,
};



#[actix_rt::main]
async fn main() {
    //529215 - 529299 : 
    //1085148		00768944084			BETHAN SARAH COLLINGRIDGE	BETHAN SARAH	COLLINGRIDGE						true
    //45623854
    let result = ipfs_host::functions::select_from_ipfs_generate_guest_input(
        "Qmdro5YY2inaDSye3vgx7nuLtcNga45kbWB11tvmW1Qx74", 
        
        (262158 * 2) - 1000, 
        (262158 * 2) + 1000,
        
    ).await;
    let start_time = std::time::Instant::now();
    let input_vec = &to_vec(&result).unwrap();
    println!("InputLength {}", input_vec.len());
    let env = ExecutorEnv::builder().add_input(input_vec).build().unwrap();
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, VERIFY_IPFS_CONTENT_ELF).unwrap();
    let elapsed_time = start_time.elapsed();
    println!("Processing time: {} ms", elapsed_time.as_millis());
//     let a = result.unwrap();
//     let b = a.2;
// //     ipfs_core::functions::prepare_proof(
//          println!("{:?}", str::from_utf8(a.0.as_slice()));
//     vec!["QmdGA4JARsegHKRey2t4neCbM8hb4aDcu9YfsEzb5JK76E".to_string(), "subfolder2".to_string(), "subfolder2.2".to_string(), "parties.tsv".to_string()],
// 1663870677, 166387135).await;
}
