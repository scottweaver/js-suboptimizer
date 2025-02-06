#![allow(unused)]

mod error;
mod html;
mod javascript;
use error::Error;
mod manifest;
mod prelude;
use prelude::*;

use crate::manifest::Manifest;
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
use std::path::{Path, PathBuf};
use thiserror::__private::AsDisplay;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file_path: PathBuf,

    #[arg(long)]
    package_name: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Sub-optimizing file: {}", &args.file_path.as_display());

    let file_contents = fs::read(&args.file_path)?;

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut file_contents.as_slice())?;

    let document = dom.document.clone();
    let scripts = extract_javascript(&document);

    let source_text = scripts[0].as_str();

    let (mut statements, exports) = extract_js_functions(source_text);
    let statements_mut = &mut statements;

    let manifest = Manifest::from_markup_file(&args.file_path, &args.package_name )?;

    update_script_tag(&document, statements_mut, &exports, &manifest.module_name);

    let s_handle: SerializableHandle = document.clone().into();

    let module_text = exports
        .iter()
        .fold(String::new(), |acc, func| format!("{}\n{}\n", acc, func));

    let (mut html_file, mut js_file) = manifest.manifest_files()?;

    serialize(&mut html_file, &s_handle, SerializeOpts::default());
    js_file
        .write_all(module_text.as_bytes())
        .expect("Could not write to file");

    println!("Done");

    Ok(())
}
