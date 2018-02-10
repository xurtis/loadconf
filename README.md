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
use loadconf::Load;

#[derive(Deserialize)]
struct Config {
	var: String,
}

impl std::default::Default for Config {
	fn default () -> Config {
		Config {
			var: "Default value".to_string(),
		}
	}
}

fn main () {
	let config = Config.load("filename");
}
```
