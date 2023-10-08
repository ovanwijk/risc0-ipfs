
use ipfs_messages::messages;
use ipfs_core::ipfsmessages;

pub fn from_proto_link(proto: messages::PbLink) -> ipfsmessages::Risc0PbLink {
    let cloned = proto.clone();
    ipfsmessages::Risc0PbLink {
        hash: cloned.hash,
        name: cloned.name,
        tsize: cloned.tsize
    }
}
pub fn from_proto_node(proto: messages::PbNode) -> ipfsmessages::Risc0PbNode {
    let cloned = proto.clone();
    let links = cloned.links.into_iter().map(from_proto_link).collect();
    ipfsmessages::Risc0PbNode {
        links: links,
        data: cloned.data,
    }
}


pub fn from_proto_unix_time(proto: Option<messages::UnixTime>) -> Option<ipfsmessages::Risc0UnixTime> {
    match proto {
        Some(unix_time) => {
            Some(ipfsmessages::Risc0UnixTime {
                seconds: unix_time.seconds,
                fractional_nanoseconds: unix_time.fractional_nanoseconds,
            })
        },
        None => None,
    }
}



pub fn from_proto_data(proto: messages::Data) -> ipfsmessages::Risc0Data {
    let cloned = proto.clone();
    ipfsmessages::Risc0Data {
        r#type: cloned.r#type,
        data: cloned.data,
        filesize: cloned.filesize,
        blocksizes: cloned.blocksizes,
        hash_type: cloned.hash_type,
        fanout: cloned.fanout,
        mode: cloned.mode,
        mtime: from_proto_unix_time(cloned.mtime)
    }
}


