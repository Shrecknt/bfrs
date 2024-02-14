# BFRS

A 🚀 blazingly fast 🚀 brainfuck optimizer, interpreter, and compiler, written in Rust.

```
Usage: bfrs [OPTIONS] --input <TARGET>

Options:
  -i, --input <TARGET>           Brainfuck file to compile or interpret
  -O <OPTIMIZATION>              Optimization level from 0 to 1 where 0 is not optimized at all and 1 is highly optimized `default = 0`
  -m, --memory <MEMORY_SIZE>     Maximum amount of memory that the brainfuck program can use `default = 2048`
  -c, --chunk-size <CHUNK_SIZE>  Size of chunks to be used in the parsing pipeline `default = 64`
  -h, --help                     Print help
```