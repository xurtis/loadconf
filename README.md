A simple library for loading configuration files from disk. All that's
required is a struct with `serde::Deserialize` and `Default` implemented.

The configuration file is always assumed to be encoded in TOML format.

The library will load the first struct it finds in the following list:

1. `./{name}`
1. `./{name}.toml`
1. `./.{name}`
1. `./.{name}.toml`
1. `~/.{name}`
1. `~/.{name}.toml`
1. `~/.config/{name}`
1. `~/.config/{name}.toml`
1. `~/.config/{name}/config`
1. `~/.config/{name}/config.toml`
1. `/etc/.config/{name}`
1. `/etc/.config/{name}.toml`
1. `/etc/.config/{name}/config`
1. `/etc/.config/{name}/config.toml`

# Usage

```rust
#[macro_use]
extern crate serde_derive;
extern crate loadconf;

/// Sample configuration
#[derive(Deserialize)]
struct Config {
    /// Sample variable
    var: String,
}

impl Default for Config {
    fn default() -> Config {
        Config { var: "Test configuration.".to_string() }
    }
}

fn main() {
    use loadconf::Load;

    let config = Config::load("testcfg");
}
```
