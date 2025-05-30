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
}
