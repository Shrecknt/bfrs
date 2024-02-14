use bfrs::{collect::Execute, deduplicate::DeduplicatedAstNode};
use clap::Parser;
use std::{
    fs,
    io::Cursor,
    sync::mpsc::{channel, Receiver},
    thread,
    time::Instant,
};

const DEFAULT_OPTIMIZATION: usize = 0;
const DEFAULT_MEMORY_SIZE: usize = 2048;
const DEFAULT_CHUNK_SIZE: usize = 8;

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
    /// `default = 8`
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
            let receiver = token_channel.1;
            bfrs::ast::AstNode::parse(&receiver, ast_channel.0);
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
    println!("{collected:?}");
    if args.optimization.unwrap_or(DEFAULT_OPTIMIZATION) >= 1 {
        collected = bfrs::optimizations::collapse_unbounded(collected);
    }

    let end_time = Instant::now();
    let duration = end_time - start_time;
    println!("Parsed AST in {duration:?}");

    println!("{collected:?}");

    let mut state = vec![0_u8; args.memory_size.unwrap_or(DEFAULT_MEMORY_SIZE)];
    let mut pointer = 0_isize;

    collected.execute(state.as_mut_slice(), &mut pointer);

    // print_ast(deduplicate_channel.1, 0);
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
