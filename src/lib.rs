//! This library makes it simple to load configuration files from disk.
//!
//! Configuration can be loaded into any struct that implements
//! `serde::Deserialize` and `std::default::Default`.
//!
//! The library will load the first struct it finds in the following list,
//! falling back to the default if no files are found.
//!
//! 1. `./{name}`
//! 1. `./{name}.toml`
//! 1. `./.{name}`
//! 1. `./.{name}.toml`
//! 1. `~/.{name}`
//! 1. `~/.{name}.toml`
//! 1. `~/.config/{name}`
//! 1. `~/.config/{name}.toml`
//! 1. `~/.config/{name}/config`
//! 1. `~/.config/{name}/config.toml`
//! 1. `/etc/.config/{name}`
//! 1. `/etc/.config/{name}.toml`
//! 1. `/etc/.config/{name}/config`
//! 1. `/etc/.config/{name}/config.toml`
//!
//! # Example Usage
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate loadconf;
//!
//! /// Sample configuration
//! #[derive(Deserialize)]
//! struct Config {
//!     /// Sample variable
//!     var: String,
//! }
//!
//! impl Default for Config {
//!     fn default() -> Config {
//!         Config { var: "Test configuration.".to_string() }
//!     }
//! }
//!
//! fn main() {
//!     use loadconf::Load;
//!
//!     // Just search for configuration files
//!     let config = Config::load("sample");
//!
//!     // Optionally use file specified on command line.
//!     use std::env;
//!     let mut args = env::args();
//!     args.next();
//!     let config = Config::fallback_load("sample", args.next());
//! }
//! ```

extern crate serde;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[allow(unused_imports)]
extern crate tempdir;
extern crate toml;

mod error;

pub use error::Error;
use serde::de::DeserializeOwned;
use std::default::Default;
use std::env::home_dir;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Load a struct from a configuration file.
pub trait Load: Sized {
    /// Find a configuration file and load the contents, falling back to
    /// the default.
    ///
    /// # Panics
    ///
    /// This will panic if there are any issues reading or deserializing the file.
    /// To catch these errors, use `try_load` instead.
    fn load<S: AsRef<str>>(filename: S) -> Self {
        Load::try_load(filename).expect("Error reading configuration from file")
    }

    /// Find a configuration file and load the contents, falling back to
    /// the default. Errors if file can't be read or deserialized.
    fn try_load<S: AsRef<str>>(filename: S) -> Result<Self, Error> {
        Load::try_fallback_load::<S, &str>(filename, None)
    }

    /// Loads the configuration from the given path or falls back to search if
    /// the path is None.
    ///
    /// # Panics
    ///
    /// This will panic if there are any issues reading or deserializing the file.
    /// To catch these errors, use `try_load` instead.
    fn fallback_load<S: AsRef<str>, P: AsRef<Path>>(filename: S, path: Option<P>) -> Self {
        Load::try_fallback_load(filename, path).expect("Error reading configuration from file")
    }

    /// Loads the configuration from the given path or falls back to search if
    /// the path is None. Errors if file can't be read or deserialized.
    fn try_fallback_load<S: AsRef<str>, P: AsRef<Path>>(filename: S, path: Option<P>) -> Result<Self, Error>;
}

impl<C> Load for C
where
    C: Default + DeserializeOwned,
{
    fn try_fallback_load<S: AsRef<str>, P: AsRef<Path>>(filename: S, path: Option<P>) -> Result<C, Error> {
        if let Some(path) = path {
            read_from_file(path.as_ref())
        } else {
            let paths = path_list(filename.as_ref());

            match paths.iter().find(|p| p.exists()) {
                Some(path) => read_from_file(path),
                None => Ok(Default::default()),
            }
        }
    }
}

/// Read a configuration from a file.
fn read_from_file<P, C>(path: P) -> Result<C, Error>
where
    P: AsRef<Path>,
    C: Default + DeserializeOwned,
{
    let mut text = String::new();
    File::open(path)?.read_to_string(&mut text)?;
    Ok(toml::from_str(&text)?)
}

/// Generate a vector of all the paths to search for a configuration file.
fn path_list(name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Add relative paths
    let mut relative_paths = vec![
        format!("{}", name),
        format!("{}.toml", name),
        format!(".{}", name),
        format!(".{}.toml", name),
    ];
    paths.append(&mut relative_paths);

    // Get the home directory as a string.
    let home = home_dir()
        .map(|h| h.into_os_string())
        .and_then(|p| p.into_string().ok());

    // Add home paths
    let mut home_paths = match home {
        Some(home) => {
            vec![
                format!("{}/.{}", home, name),
                format!("{}/.{}.toml", home, name),
                format!("{}/.config/{}", home, name),
                format!("{}/.config/{}.toml", home, name),
                format!("{}/.config/{}/config", home, name),
                format!("{}/.config/{}/config.toml", home, name),
            ]
        }
        None => vec![],
    };
    paths.append(&mut home_paths);

    // Add absolute paths
    let mut absolute_paths = vec![
        format!("/etc/.config/{}", name),
        format!("/etc/.config/{}.toml", name),
        format!("/etc/.config/{}/config", name),
        format!("/etc/.config/{}/config.toml", name),
    ];
    paths.append(&mut absolute_paths);

    paths
        .into_iter()
        .map(|p| AsRef::<Path>::as_ref(&p).to_path_buf())
        .collect()
}

#[cfg(test)]
mod test {

    /// Sample configuration
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Config {
        /// Sample variable
        var: String,
    }

    impl Default for Config {
        fn default() -> Config {
            Config { var: "Test configuration.".to_string() }
        }
    }

    /// Test that path list produces the expected path list.
    #[test]
    fn generate_path_list() {
        use std::env::set_var;
        use std::path::{Path, PathBuf};

        set_var("HOME", "/home/test");
        let paths = super::path_list("testcfg");

        let expected: Vec<PathBuf> = vec![
            "testcfg",
            "testcfg.toml",
            ".testcfg",
            ".testcfg.toml",
            "/home/test/.testcfg",
            "/home/test/.testcfg.toml",
            "/home/test/.config/testcfg",
            "/home/test/.config/testcfg.toml",
            "/home/test/.config/testcfg/config",
            "/home/test/.config/testcfg/config.toml",
            "/etc/.config/testcfg",
            "/etc/.config/testcfg.toml",
            "/etc/.config/testcfg/config",
            "/etc/.config/testcfg/config.toml",
        ].into_iter()
            .map(|p| AsRef::<Path>::as_ref(&p).to_path_buf())
            .collect();

        assert_eq!(paths, expected);
    }

    /// Test loading default configuration.
    #[test]
    fn load_default() {
        use super::Load;

        let config = Config::load("testcfg");
        assert_eq!(config, Config::default());
    }

    /// Test load configuration from a file.
    #[test]
    fn file_test() {
        use std::env::set_current_dir;
        use std::fs::OpenOptions;
        use std::io::Write;
        use super::Load;
        use tempdir::TempDir;

        // Change into temporary testi directory
        let temp_dir = TempDir::new("loadcfg-test")
            .expect("Could not create temporary directory for test");
        set_current_dir(temp_dir.path())
            .expect("Could not change into temporary directory for test");

        // Write the test configuration file.
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(".testcfg.toml")
            .expect("Couldn't open test configuration file.")
            .write_all("var = \"Test configuration file\"\n".as_bytes())
            .expect("Couldn't write test configuration file.");

        // Load the file
        let config = Config::load("testcfg");
        let expected = Config { var: "Test configuration file".to_string() };
        assert_eq!(config, expected);
    }

    /// Test load configuration from a file specified directly.
    #[test]
    fn specified_file_test() {
        use std::env::set_current_dir;
        use std::fs::OpenOptions;
        use std::io::Write;
        use super::Load;
        use tempdir::TempDir;

        // Change into temporary testi directory
        let temp_dir = TempDir::new("loadcfg-test")
            .expect("Could not create temporary directory for test");
        set_current_dir(temp_dir.path())
            .expect("Could not change into temporary directory for test");

        // Write the test configuration file.
        OpenOptions::new()
            .write(true)
            .create(true)
            .open("given-config.toml")
            .expect("Couldn't open test configuration file.")
            .write_all("var = \"Test configuration file\"\n".as_bytes())
            .expect("Couldn't write test configuration file.");

        // Load the file
        let config = Config::fallback_load("testcfg", Some("given-config.toml"));
        let expected = Config { var: "Test configuration file".to_string() };
        assert_eq!(config, expected);
    }
}
