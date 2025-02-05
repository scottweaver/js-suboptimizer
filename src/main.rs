use clap::Parser;
use kuchikiki::traits::*;
use kuchikiki::{parse_html, NodeRef};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Function, Program};
use oxc_ast::visit::Visit;
use oxc_ast::*;
use oxc_minifier::*;
use oxc_parser::{ParseOptions, Parser as OxcParser};
use oxc_span::SourceType;
use std::any::Any;
use std::fmt::Display;
use std::fs;

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
    let file_contents = fs::read_to_string(args.file_name).unwrap();
    let document = parse_html().one(file_contents);

    // fn walk(node: &NodeRef, depth: usize) {
    //     let indent = "  ".repeat(depth);
    //     println!("{}{:?}", indent, node);
    //
    //     node.children().for_each(|child| {
    //         walk(&child, depth + 1);
    //     });
    // }

    fn extract_javascript(document: &NodeRef) -> Vec<String> {
        let script_elements = document.select("script").unwrap();
        let scripts: Vec<String> = script_elements
            .map(|script| {
                let text = script.text_contents();
                text
            })
            .collect();
        scripts
    }

    // fn print_all(scripts: &Vec<String>) {
    //     scripts.iter().for_each(|script| {
    //         println!("{}", script);
    //     });
    // }

    // #[derive(Debug, Default)]
    // struct Visitor {
    //     split_locs: Vec<u32>,
    // }
    // ;
    // 
    // impl Visitor {
    //     fn new(split_locs: Vec<u32>) -> Self {
    //         Self { split_locs }
    //     }
    // }
    // 
    // impl Visit<'_> for Visitor {
    //     fn visit_function(&mut self, it: &Function<'_>, flags: oxc_syntax::scope::ScopeFlags) {
    //         let s = it.span.start;
    //         let e = it.span.end;
    //         let split_offset = e - 1;
    //         if (self.split_locs.contains(&split_offset)) {
    //             println!(
    //                 "[Extracting Function]: '' {:?} starts at {} and ends at {}",
    //                 it.name().unwrap().as_str(),
    //                 s,
    //                 e
    //             );
    //         } else {
    //             println!(
    //                 "[Ignoring function]: {:?} starts at {} and ends at {}",
    //                 it.name().unwrap().as_str(),
    //                 s,
    //                 e
    //             );
    //         }
    //     }
    // 
    // }
    // 
    // let allocator = Allocator::default();
    // let source_type = SourceType::cjs();

    let scripts = extract_javascript(&document);

    let source_text = scripts[0].as_str();


    println!("---------------------------------------------------");

    #[derive(Debug, PartialOrd, PartialEq)]
    enum State {
        Scanning,
        JsFunction {
            name: Option<String>,
            tokens: Vec<String>,
            depth: u32,
            split: bool,
        },
        SingleComment,
        Splitting,
    }

    impl Display for State {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                State::Scanning => write!(f, " "),
                State::JsFunction {
                    name,
                    tokens,
                    depth,
                    split,
                } => {
                    let c0 = if *split { "export " } else { "" };
                    let s: String = tokens.iter().fold(String::new(), |acc, token| acc + token);
                    write!(f, "{}{}", c0, s)
                }
                State::SingleComment => write!(f, "//"),
                State::Splitting => write!(f, ""),
            }
        }
    }

    fn restore_ws(token: &str, depth: u32) -> String {
        let p0 = depth;
        let padding = "  ".repeat(p0 as usize);
        if (token.ends_with("{")) {
            token.to_owned() + "\n" + padding.as_str()
        } else if token.ends_with(";") {
            token.to_owned() + "\n"
        } else if token.ends_with("}") {
            padding + token + "\n"
        } else {
            token.to_owned() + " "
        }
    }

    fn parseFuncs(source_text: &str) -> (Vec<State>, Vec<State>) {
        let tokens = source_text.split_ascii_whitespace();
        let mut exports: Vec<State> = Vec::new();

        let staying: Vec<State> = tokens
            .fold(
                (Vec::<State>::new(), State::Scanning),
                |(mut buffer, state), token| match state {
                    State::Splitting => {
                        if token == "function" {
                            (
                                buffer,
                                State::JsFunction {
                                    name: None,
                                    tokens: Vec::from([restore_ws(token, 1)]),
                                    depth: 0,
                                    split: true,
                                },
                            )
                        } else {
                            (buffer, State::Splitting)
                        }
                    }
                    State::SingleComment => {
                        if token == "split" {
                            (buffer, State::Splitting)
                        } else {
                            (buffer, State::Scanning)
                        }
                    }
                    State::Scanning => {
                        if token == "function" {
                            (
                                buffer,
                                State::JsFunction {
                                    name: None,
                                    tokens: Vec::from([restore_ws(token, 1)]),
                                    depth: 0,
                                    split: false,
                                },
                            )
                        } else if token == "//" {
                            (buffer, State::SingleComment)
                        } else {
                            (buffer, State::Scanning)
                        }
                    }
                    State::JsFunction {
                        name,
                        mut tokens,
                        depth,
                        split,
                    } => {
                        let name0 = match name {
                            Some(name) => Some(name),
                            None if tokens.len() == 2 => {
                                let name =
                                    // Is there a better way to do this?
                                    tokens[1].split("(").collect::<Vec<&str>>()[0].trim().to_string();
                                Some(name)
                            }
                            None => None,
                        };

                        if token == "{" {
                            tokens.push(restore_ws(token, depth));
                            (
                                buffer,
                                State::JsFunction {
                                    name: name0,
                                    tokens,
                                    depth: depth + 1,
                                    split,
                                },
                            )
                        } else if token == "}" {
                            let depth0 = depth - 1;
                            tokens.push(restore_ws(token, depth0));
                            if depth0 > 0 {
                                (
                                    buffer,
                                    State::JsFunction {
                                        name: name0,
                                        tokens,
                                        depth: depth0,
                                        split,
                                    },
                                )
                            } else {
                                if split {
                                    exports.push(State::JsFunction {
                                        name: name0,
                                        tokens,
                                        depth,
                                        split,
                                    });
                                } else {
                                    buffer.push(State::JsFunction {
                                        name: name0,
                                        tokens,
                                        depth: 0,
                                        split,
                                    });
                                }

                                (buffer, State::Scanning)
                            }
                        } else {
                            tokens.push(restore_ws(token, depth));
                            (
                                buffer,
                                State::JsFunction {
                                    name: name0,
                                    tokens,
                                    depth,
                                    split,
                                },
                            )
                        }
                    }
                },
            )
            .0;
        (staying, exports)
    }

    let (funcs, exports) = parseFuncs(source_text);

    funcs.iter().for_each(|func| {
        println!("{}", func);
    });

    println!("---------------------EXPORTS--------------------------");
    println!("{:?}", exports);


    println!("Done");
}
