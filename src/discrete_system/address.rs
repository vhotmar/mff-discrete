use serde::{Deserialize, Serialize};

pub type Address = u32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddressGenerator {
    curr: u32,
}

/// Original thought was be able to parallelize the computation, so there
/// had to be unique IDs across threads

impl AddressGenerator {
    pub fn new() -> AddressGenerator {
        AddressGenerator { curr: 0 }
    }

    pub fn next(&mut self) -> Address {
        let addr = self.curr;

        self.curr += 1;

        addr
    }
}