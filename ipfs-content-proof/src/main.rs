// Import the core library
extern crate ipfs_host;
use std::str;
#[actix_rt::main]
async fn main() {
    //529215 - 529299 : 
    //1085148		00768944084			BETHAN SARAH COLLINGRIDGE	BETHAN SARAH	COLLINGRIDGE						true
    //45623854
    let result = ipfs_host::functions::depth_first_search(
        "Qmdro5YY2inaDSye3vgx7nuLtcNga45kbWB11tvmW1Qx74", 
        0,
        (262158 * 2) - 1000, 
        (262158 * 2) + 1000,
        vec![]
    ).await;
    let a = result.unwrap();
    let b = a.2;
//     ipfs_core::functions::prepare_proof(
         println!("{:?}", str::from_utf8(a.0.as_slice()));
//     vec!["QmdGA4JARsegHKRey2t4neCbM8hb4aDcu9YfsEzb5JK76E".to_string(), "subfolder2".to_string(), "subfolder2.2".to_string(), "parties.tsv".to_string()],
// 1663870677, 166387135).await;
}
