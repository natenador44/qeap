use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex, RwLock},
};

use qeap::{
    Qeap,
    file::{FilePersist, json::Json},
};
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

static CONFIG_PERSISTENCE: LazyLock<FilePersist<Json<Config>>> =
    LazyLock::new(|| qeap::file::FilePersist::<Json<Config>>::new("test_data"));

#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(persist_with = &*CONFIG_PERSISTENCE)]
pub struct Config {
    something_something: u8,
    port: u16,
}

#[qeap::scoped(flatten)]
pub fn immut_ref(data: &Config) {
    println!("{}", data.something_something);
}

#[qeap::scoped(absorb)]
fn mut_ref(config: &mut Config) -> Result<(), MyError> {
    config.port = 8080;
    Ok(())
}

#[qeap::scoped(flatten_erased)]
fn config_rc(config: Rc<Config>) {
    println!("{}", config.port);
}

#[qeap::scoped(expect)]
fn config_rc_ref_cell(config: Rc<RefCell<Config>>) {
    let c = config.borrow();
    println!("{}", c.port);
}

#[qeap::scoped]
fn config_arc(config: Arc<Config>) {
    println!("{}", config.port);
}

#[qeap::scoped]
fn config_arc_mutex(config: Arc<Mutex<Config>>) {
    let c = config.lock().unwrap();
    println!("{}", c.port);
}

#[qeap::scoped]
fn config_arc_rwlock(config: Arc<RwLock<Config>>) {
    let c = config.read().unwrap();
    println!("{}", c.port);
}
