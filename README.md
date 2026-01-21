# Godot client rust code

In-Its-Brilliance/.cargo/config.toml

``` toml
[patch."https://github.com/In-Its-Brilliance/brilliance-common"]
common = { path = "brilliance-common" }

[patch."https://github.com/In-Its-Brilliance/brilliance-network"]
network = { path = "brilliance-network" }

[patch."https://github.com/In-Its-Brilliance/brilliance-physics"]
physics = { path = "brilliance-physics" }
```

## Run tests
```shell
cargo test
```

## Build debug rust bynary for Linux and Windows
```shell
cargo build
```
## Build project for Linux
```shell
godot --export-release linux_desktop ~/godot/Brilliance.x86_64
```

## Build release for Windows from Linux
```shell
rustup target add x86_64-pc-windows-gn
cargo build --release --target x86_64-pc-windows-gnu

godot --export-release windows_desktop ~/Dropbox/Brilliance/Brilliance.exe
```

## Run game from the console

```shell
godot --path ./ ./scenes/main_menu.tscn
```
