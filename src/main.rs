#![allow(unused)]

mod html;
mod javascript;

use clap::Parser;
use html::*;
use html5ever::serialize::{serialize, SerializeOpts};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, ParseOpts};
use javascript::*;
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_name: String,

    #[arg(long)]
    html_output_file: String,

    #[arg(long)]
    module_output_file: String,
}

fn main() {
    let args = Args::parse();

    println!("Opening file: {}", args.file_name);

    let file_contents = fs::read(args.file_name).unwrap();

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut file_contents.as_slice())
        .unwrap();

    let document = dom.document.clone();
    let scripts = extract_javascript(&document);

    let source_text = scripts[0].as_str();

    println!("---------------------------------------------------");

    let (mut statements, exports) = extract_js_functions(source_text);
    let statements_mut = &mut statements;


    println!("---------------------UPDATES BEFORE--------------------------");
    println!("{:?}", statements_mut);

    update_script_tag_rec(&document, statements_mut, &exports, &args.module_output_file);


    println!("---------------------EXPORTS--------------------------");
    println!("{:?}", exports);

    println!("---------------------UPDATES AFTER--------------------------");
    println!("{:?}", statements_mut);

    let s_handle: SerializableHandle = document.clone().into();

    let module_text = exports.iter().fold(String::new(), |acc, func| {
        format!("{}\n{}\n", acc, func)
    });
 
    let mut html_file = File::create(args.html_output_file).expect("Could not create file");
    let mut js_file = File::create(args.module_output_file).expect("Could not create file");

    serialize(&mut html_file, &s_handle, SerializeOpts::default());
    js_file.write_all(module_text.as_bytes()).expect("Could not write to file");

    println!("Done");
}
