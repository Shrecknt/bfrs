use bfrs::{collect::Execute, deduplicate::DeduplicatedAstNode, ChunkedReceiverWrapper};
use clap::Parser;
use std::{
    fs,
    io::Cursor,
    sync::mpsc::{channel, Receiver},
    thread,
};

#[cfg(debug_assertions)]
use std::time::Instant;

const DEFAULT_OPTIMIZATION: usize = 0;
const DEFAULT_MEMORY_SIZE: usize = 2048;
const DEFAULT_CHUNK_SIZE: usize = 64;

#[derive(Parser, Debug)]
struct Args {
    /// Brainfuck file to compile or interpret
    #[arg(short = 'i', long = "input", value_hint = clap::ValueHint::DirPath)]
    pub target: std::path::PathBuf,

    /// Optimization level from 0 to 1
    /// where 0 is not optimized at all
    /// and 1 is highly optimized
    /// `default = 0`
    #[arg(short = 'O')]
    pub optimization: Option<usize>,

    /// Maximum amount of memory that
    /// the brainfuck program can use
    /// `default = 2048`
    #[arg(short = 'm', long = "memory")]
    pub memory_size: Option<usize>,

    /// Size of chunks to be used in
    /// the parsing pipeline
    /// `default = 64`
    #[arg(short = 'c', long = "chunk-size")]
    pub chunk_size: Option<usize>,
}

fn main() {
    let args: Args = Args::parse();

    let token_channel = channel();
    let ast_channel = channel();
    let deduplicate_channel = channel();

    let file = fs::read(args.target).expect("Input file not found");
    let mut program = Cursor::new(file.as_slice());

    #[cfg(debug_assertions)]
    let start_time = Instant::now();

    let mut collected = vec![];
    thread::scope(|s| {
        s.spawn(|| {
            bfrs::parse::Token::parse(
                &mut program,
                token_channel.0,
                args.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            );
        });
        s.spawn(|| {
            let mut receiver = ChunkedReceiverWrapper::new(token_channel.1);
            bfrs::ast::AstNode::parse(
                &mut receiver,
                ast_channel.0,
                args.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            );

            #[cfg(debug_assertions)]
            {
                let end_time = Instant::now();
                let duration = end_time - start_time;
                println!("Parsed AST in {duration:?}");
            }
        });
        s.spawn(|| {
            let receiver = ast_channel.1;
            bfrs::deduplicate::DeduplicatedAstNode::parse(receiver, deduplicate_channel.0);
        });
        s.spawn(|| {
            let receiver = deduplicate_channel.1;
            collected = bfrs::collect::CollectedAstNode::parse(&receiver);
        });
    });
    if args.optimization.unwrap_or(DEFAULT_OPTIMIZATION) >= 1 {
        collected = bfrs::optimizations::collapse_unbounded(collected);
    }

    #[cfg(debug_assertions)]
    {
        let end_time = Instant::now();
        let duration = end_time - start_time;
        println!("Parsed and optimized AST in {duration:?}");
    }

    let mut state = vec![0_u8; args.memory_size.unwrap_or(DEFAULT_MEMORY_SIZE)];
    let mut pointer = 0_isize;

    #[cfg(debug_assertions)]
    let start_time = Instant::now();
    collected.execute(state.as_mut_slice(), &mut pointer);
    #[cfg(debug_assertions)]
    {
        let end_time = Instant::now();
        let duration = end_time - start_time;
        println!("Execution took {duration:?}");
    }
}

#[allow(unused)]
fn print_ast(receiver: Receiver<Vec<DeduplicatedAstNode>>, indent: usize) {
    let indent_str = "    ".repeat(indent);
    while let Ok(nodes) = receiver.recv() {
        for node in nodes {
            match node {
                DeduplicatedAstNode::Group(r) => {
                    println!("{indent_str}Group [");
                    print_ast(r, indent + 1);
                    println!("{indent_str}]");
                }
                _ => println!("{indent_str}{node:?}"),
            }
        }
    }
}
