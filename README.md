# Godot client rust code

## Release build
``` shell
cargo build --release --no-default-features --features use-git,trace
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
