use bfrs::{collect::CollectedAstNode, deduplicate::DeduplicatedAstNode};
use std::{
    io::{Cursor, Read},
    sync::mpsc::{channel, Receiver},
    thread,
    time::Instant,
};

fn main() {
    let token_channel = channel();
    let ast_channel = channel();
    let deduplicate_channel = channel();

    let mut program = Cursor::new(include_bytes!("../lost_king.bf").as_slice());

    let start_time = Instant::now();

    thread::scope(|s| {
        s.spawn(|| {
            bfrs::parse::Token::parse(&mut program, token_channel.0);
        });
        s.spawn(|| {
            let receiver = token_channel.1;
            bfrs::ast::AstNode::parse(&receiver, ast_channel.0);
        });
        s.spawn(|| {
            let receiver = ast_channel.1;
            bfrs::deduplicate::DeduplicatedAstNode::parse(receiver, deduplicate_channel.0);
        });
    });

    let end_time = Instant::now();
    let duration = end_time - start_time;
    println!("Parsed AST in {duration:?}");

    let collected = CollectedAstNode::parse(&deduplicate_channel.1);
    println!("{collected:?}");

    let mut state = [0_u8; 2048];
    let mut pointer = 0_isize;

    execute_bf(&mut state, &mut pointer, &collected, false);

    // print_ast(deduplicate_channel.1, 0);
}

fn execute_bf(
    state: &mut [u8; 2048],
    pointer: &mut isize,
    program: &Vec<CollectedAstNode>,
    is_group: bool,
) {
    if is_group && state[*pointer as usize] == 0 {
        return;
    }
    loop {
        for node in program {
            match node {
                CollectedAstNode::ModifyPointer(val) => *pointer += *val,
                CollectedAstNode::ModifyData(val) => state[*pointer as usize] += *val as u8,
                CollectedAstNode::Output => print!("{}", state[*pointer as usize] as char),
                CollectedAstNode::Input => {
                    state[*pointer as usize] = std::io::stdin().bytes().next().unwrap().unwrap()
                }
                CollectedAstNode::Group(program) => execute_bf(state, pointer, program, true),
            }
        }
        if state[*pointer as usize] == 0 || !is_group {
            return;
        }
    }
}

#[allow(unused)]
fn print_ast(receiver: Receiver<DeduplicatedAstNode>, indent: usize) {
    let indent_str = "    ".repeat(indent);
    while let Ok(node) = receiver.recv() {
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
