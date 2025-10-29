use qeap::Qeap;
use serde::{Deserialize, Serialize};

#[allow(unused)]
enum MyError {
    Bad(String),
    Qeap(qeap::error::Error),
}

impl From<qeap::error::Error> for MyError {
    fn from(value: qeap::error::Error) -> Self {
        MyError::Qeap(value)
    }
}

#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(dir = "test_data")]
pub struct Config {
    something_something: u8,
    port: u16,
}

#[qeap::scoped_test(mode = expect, expected_ret_pat = 0)]
fn return_u16_works(config: &Config) -> u16 {
    config.port
}
