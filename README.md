# This is a simple CHIP-8 emulator written in Rust.

## How to build

```
cargo build 
```

## How to use

fire up the binary with the `-p` flag to specify the path to a CHIP-8 ROM
```
chip8 -p path/to/rom
```

use the `-f` flag to set the instructions per second
```
chip8 -p path/to/rom -f 354
```

use `--help` to see a detailed description of all available commands
```
chip8 --help
```