# tlenek

A basic x86_64 OS.

Made with help from [Writing an OS in Rust](https://os.phil-opp.com/) by Philipp Oppermann.

## Usage - QEMU

```bash
qemu-system-x86_64 -drive format=raw,file=path/to/binary/bootimage-tlenek.bin
```
