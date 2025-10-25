use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use qeap::Qeap;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Qeap)]
#[qeap(dir = "test_data")]
pub struct Config {
    something_something: u8,
    port: u16,
}

#[qeap::scoped]
pub fn immut_ref(data: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", data.something_something);
    Ok(())
}

#[qeap::scoped]
fn mut_ref(config: &mut Config) -> qeap::QeapResult<()> {
    config.port = 8080;
    Ok(())
}

#[qeap::scoped]
fn config_rc(config: Rc<Config>) -> qeap::QeapResult<()> {
    println!("{}", config.port);
    Ok(())
}

#[qeap::scoped]
fn config_rc_ref_cell(config: Rc<RefCell<Config>>) -> qeap::QeapResult<()> {
    let c = config.borrow();
    println!("{}", c.port);
    Ok(())
}

#[qeap::scoped]
fn config_arc(config: Arc<Config>) -> qeap::QeapResult<()> {
    println!("{}", config.port);
    Ok(())
}

#[qeap::scoped]
fn config_arc_mutex(config: Arc<Mutex<Config>>) -> qeap::QeapResult<()> {
    let c = config.lock().unwrap();
    println!("{}", c.port);
    Ok(())
}

#[qeap::scoped]
fn config_arc_rwlock(config: Arc<RwLock<Config>>) -> qeap::QeapResult<()> {
    let c = config.read().unwrap();
    println!("{}", c.port);
    Ok(())
}

#[cfg(test)]
mod tests {

    macro_rules! qeap_test {
        ($test_name:ident => $test:block) => {
            mod $test_name {
                use std::path::PathBuf;
                use qeap::Qeap;
                use serde::{Deserialize, Serialize};

                static TEST_DIR: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| std::env::temp_dir().join(format!("test_data/{}", stringify!($test_name))));

                #[derive(Debug, Default, Serialize, Deserialize, Qeap, PartialEq, Eq)]
                #[qeap(dir = &*TEST_DIR)]
                struct Config {
                    port: u16,
                    timeout_seconds: u8,
                    log_location: String,
                }

                #[test]
                fn $test_name() {
                    $test
                    std::fs::remove_dir_all(&*TEST_DIR).unwrap();
                }
            }
        };
    }

    qeap_test!(initial_load_returns_default_impl => {
        let actual = Config::load().unwrap();

        let expected = Config::default();

        assert_eq!(expected, actual);
    });

    qeap_test!(load_creates_file_with_name_of_type => {
        Config::load().unwrap();

        assert!(Config::file_path().exists());
    });

    qeap_test!(save_works => {
        let mut actual = Config::load().unwrap();

        actual.port = 8080;

        actual.save().expect("save works")
    });

    qeap_test!(load_after_save_reflects_changes_made_before_save => {
        let mut expected = Config::load().unwrap();

        expected.port = 8080;
        expected.log_location = "logs".into();

        expected.save().expect("save works");

        let actual = Config::load().unwrap();

        assert_eq!(expected, actual);
    });
}
