#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_code)]

#![allow(unused_mut)]

pub mod scanner;
pub mod compiler;
pub mod execution;

use std::iter::Iterator;
use std::{
    io::{self, Write},
    iter::Scan,
};
use tabled::{self, Table, Modify, Wrap, Style, Concat, builder::Builder};
use rustyline::error::ReadlineError;
use rustyline::Editor;

// let stmt: String = String::from("p0 & (p1 | p2)");
// let stmt: String = String::from("({p})");
// let stmt: String = String::from("!({p0 | {p1}} & {p2})");
// let stmt: String = String::from("(p0 or p1) and ! {p0 & p1}");

fn pow2(exponent: u32) -> u32
{
    return (2 as u32).pow(exponent);
}

struct BoolPermutationsIterator
{
    data: Vec<bool>,
    pub size: u32,
    current: u32,
    last: u32,
}

impl BoolPermutationsIterator
{
    pub fn new(size: u32) -> Self
    {
        Self {
            data: vec![false; size as usize],
            size,
            current: 0,
            last: pow2(size)
        }
    }

    pub fn get(&self) -> &Vec<bool>
    {
        return &self.data;
    }

    pub fn finished(&self) -> bool
    {
        return self.current >= self.last;
    }

    pub fn advance(&mut self)
    {
        for col in 0..self.size
        {
            self.data[col as usize] = (self.current / pow2(self.size - col - 1)) % 2 == 0;
        }
        self.current += 1;
    }
}

fn map_bool_cell(val: &bool) -> &'static str
{
    if *val { "T" } else { "F" }
}

fn process_input(stmt: String)
{
    let tokens = scanner::tokenize(&stmt);

    // empty query, only the EOF token is present
    if tokens.len() == 1 { return; }

    let compiled_result = compiler::compile(&tokens);

    if let Some(error_token) = compiled_result.error_token
    {
        println!("Error at token: \"{}\"\n", error_token.lexeme);
        return;
    }

    let groups = execution::subexpression_groups(compiled_result.root.as_ref().unwrap());
    let reprs = execution::groups_to_string(&groups, &compiled_result.variables);

    let mut iter = BoolPermutationsIterator::new(compiled_result.variables.len() as u32);
    let mut row_results = vec![false; groups.len()];

    let mut builder_input = Builder::default().set_header(&compiled_result.variables[..]);
    let mut builder_output = Builder::default().set_header(&reprs[..]);

    loop {
        iter.advance();

        let table_row_input = iter.get()
                .iter()
                .map(map_bool_cell)
                .collect::<Vec<&'static str>>();

        builder_input = builder_input.add_row(table_row_input);

        execution::evaluate(&groups, iter.get(), &mut row_results[..]);

        let table_row_output = row_results
                        .iter()
                        .map(map_bool_cell)
                        .collect::<Vec<&'static str>>();

        builder_output = builder_output.add_row(table_row_output);

        if iter.finished() {
            break;
        }
    }

    let style = Style::modern();    

    let table_input = builder_input
                    .build()
                    .with(style.clone());

    let table_output = builder_output
                    .build()
                    .with(Modify::new(tabled::Full).with(Wrap::new(20)))
                    .with(style.clone());

    let display_table = table_input.with(Concat::horizontal(table_output));

    println!("{}", display_table.to_string());
}



fn main()
{
    println!("Welcome to ttbl!");
    println!("Press <Ctrl-D> to exit\n");

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                process_input(line);
            },
            Err(ReadlineError::Interrupted) => {
                continue;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("I/O Error: {:?}", err);
                break
            }
        }
    }


}
