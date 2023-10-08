use prost::Message;

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}