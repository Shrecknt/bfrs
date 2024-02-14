# BFRS

A 🚀 blazingly fast 🚀 brainfuck optimizer, interpreter, and compiler, written in Rust.

```
Usage: bfrs [OPTIONS] --input <TARGET>

Options:
  -i, --input <TARGET>           Brainfuck file to compile or interpret
  -O <OPTIMIZATION>              Optimization level from 0 to 1 where 0 is barely any optimizations and 1 is highly optimized `default = 0`
  -m, --memory <MEMORY_SIZE>     Maximum amount of memory that the brainfuck program can use `default = 2048`
  -C, --chunk-size <CHUNK_SIZE>  Size of chunks to be used in the parsing pipeline `default = 64`
  -c                             Whether the program should be compiled instead of interpreted `default = false`
  -h, --help                     Print help
```