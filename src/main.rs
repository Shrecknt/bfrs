use bfrs::{collect::Execute, deduplicate::DeduplicatedAstNode, ChunkedReceiverWrapper};
use clap::Parser;
use std::{
    collections::VecDeque,
    fs,
    io::Cursor,
    sync::mpsc::{channel, Receiver},
    thread,
    time::Instant,
};

const DEFAULT_OPTIMIZATION: usize = 0;
const DEFAULT_MEMORY_SIZE: usize = 2048;
const DEFAULT_CHUNK_SIZE: usize = 64;

#[derive(Parser, Debug)]
struct Args {
    /// Brainfuck file to compile or interpret
    #[arg(short = 'i', long = "input", value_hint = clap::ValueHint::DirPath)]
    pub target: std::path::PathBuf,

    /// Optimization level from 0 to 1
    /// where 0 is barely any optimizations
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
    #[arg(short = 'C', long = "chunk-size")]
    pub chunk_size: Option<usize>,

    /// Whether the program should be
    /// compiled instead of interpreted
    /// `default = false`
    #[arg(short = 'c')]
    pub compile: bool,

    /// Whether the parser should be run
    /// synchronously instead of threaded
    /// (intended for debugging)
    /// causes occasional deadlocks atm
    /// `default = false`
    #[arg(long = "sync")]
    pub synchronous: bool,
}

fn main() {
    let args: Args = Args::parse();

    #[cfg(debug_assertions)]
    let display_times: bool = true;
    #[cfg(not(debug_assertions))]
    let display_times: bool = std::env::var("DISPLAY_TIMES") == Ok("1".to_string());

    let token_channel = channel();
    let ast_channel = channel();
    let deduplicate_channel = channel();

    let file = fs::read(args.target).expect("Input file not found");
    let mut program = Cursor::new(file.as_slice());

    let start_time = Instant::now();

    let sync_mutex = if args.synchronous {
        Some(std::sync::Arc::new(parking_lot::FairMutex::new(())))
    } else {
        None
    };

    let mut collected = VecDeque::new();
    thread::scope(|s| {
        s.spawn(|| {
            let sync_mutex = sync_mutex.clone();
            let lock = match &sync_mutex {
                Some(sync_mutex) => Some(std::rc::Rc::new(sync_mutex.lock())),
                None => None,
            };

            bfrs::parse::Token::parse(
                &mut program,
                token_channel.0,
                args.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            );

            drop(lock);
        });
        s.spawn(|| {
            let sync_mutex = sync_mutex.clone();
            let lock = match &sync_mutex {
                Some(sync_mutex) => Some(std::rc::Rc::new(sync_mutex.lock())),
                None => None,
            };

            let mut receiver = ChunkedReceiverWrapper::new(token_channel.1);
            bfrs::ast::AstNode::parse(
                &mut receiver,
                ast_channel.0,
                args.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE),
            );

            drop(lock);

            if display_times {
                let end_time = Instant::now();
                let duration = end_time - start_time;
                println!("Parsed AST in {duration:?}");
            }
        });
        s.spawn(|| {
            let sync_mutex = sync_mutex.clone();
            let lock = match &sync_mutex {
                Some(sync_mutex) => Some(std::rc::Rc::new(sync_mutex.lock())),
                None => None,
            };

            let receiver = ChunkedReceiverWrapper::new(ast_channel.1);
            bfrs::deduplicate::DeduplicatedAstNode::parse(receiver, deduplicate_channel.0);

            drop(lock);
        });
        s.spawn(|| {
            let sync_mutex = sync_mutex.clone();
            let lock = match &sync_mutex {
                Some(sync_mutex) => Some(std::rc::Rc::new(sync_mutex.lock())),
                None => None,
            };

            let mut receiver = ChunkedReceiverWrapper::new(deduplicate_channel.1);
            collected = bfrs::collect::CollectedAstNode::parse(&mut receiver);

            drop(lock);
        });
    });
    if args.optimization.unwrap_or(DEFAULT_OPTIMIZATION) >= 1 {
        collected = bfrs::optimizations::collapse_unbounded(collected);
    }

    if display_times {
        let end_time = Instant::now();
        let duration = end_time - start_time;
        println!("Parsed and optimized AST in {duration:?}");
    }

    if !args.compile {
        let mut state = vec![0_u8; args.memory_size.unwrap_or(DEFAULT_MEMORY_SIZE)];
        let mut pointer = 0_isize;

        let start_time = Instant::now();
        collected.execute(state.as_mut_slice(), &mut pointer);
        if display_times {
            let end_time = Instant::now();
            let duration = end_time - start_time;
            println!("Execution took {duration:?}");
        }
        return;
    }

    todo!()
}

#[allow(unused)]
fn print_ast(receiver: Receiver<VecDeque<DeduplicatedAstNode>>, indent: usize) {
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
