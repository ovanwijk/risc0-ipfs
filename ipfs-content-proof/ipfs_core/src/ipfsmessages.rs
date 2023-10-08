
#[derive(Clone)]
pub struct Risc0PbLink {
    /// multihash of the target object
   
    pub hash: ::core::option::Option<Vec<u8>>,
    /// utf string name. should be unique per object
    pub name: ::core::option::Option<String>,
    /// cumulative size of target object
    pub tsize: ::core::option::Option<u64>,
}
/// An IPFS MerkleDAG Node
#[derive(Clone)]
pub struct Risc0PbNode {
   
    pub links: Vec<Risc0PbLink>,
    pub data: ::core::option::Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct Risc0Data {
    
    pub r#type: ::core::option::Option<i32>,
    pub data: ::core::option::Option<Vec<u8>>,
    pub filesize: ::core::option::Option<u64>,
    pub blocksizes: Vec<u64>,
    pub hash_type: ::core::option::Option<u64>,
    pub fanout: ::core::option::Option<u64>,
    pub mode: ::core::option::Option<u32>,
    pub mtime: ::core::option::Option<Risc0UnixTime>,
}

#[derive(Clone)]
pub struct Risc0UnixTime {
    pub seconds: Option<i64>,
    pub fractional_nanoseconds: Option<u32>,
}
#[derive(Clone)]
pub struct Risc0Metadata {    
    pub mime_type: Option<String>,
}
