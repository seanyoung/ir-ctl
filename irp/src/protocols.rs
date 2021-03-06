use quick_xml::de::from_str;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
struct Protocols {
    protocol: Vec<Protocol>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub irp: String,
}

#[allow(dead_code)]
pub fn read_protocols(path: &Path) -> Vec<Protocol> {
    let protocols_xml = std::fs::read_to_string(path).expect("file not found!");

    let protocols: Protocols = from_str(&protocols_xml).expect("unexpected xml");

    protocols.protocol
}
