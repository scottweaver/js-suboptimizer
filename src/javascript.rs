use std::fmt::{Display, Formatter};

#[derive(Debug, PartialOrd, PartialEq)]
enum State {
    Statement,
    ExportJsFunction {
        name: Option<String>,
        tokens: Vec<String>,
        depth: u32,
        split: bool,
    },
    SingleComment,
    Splitting,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct JsStatement {
    tokens: Vec<String>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ExportJsFunction {
    pub name: Option<String>,
    pub tokens: Vec<String>,
}

impl From<&JsStatement> for String {
    fn from(js_function: &JsStatement) -> String {
        format!("{}", js_function)
    }
}

impl Display for ExportJsFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .tokens
            .iter()
            .fold(String::new(), |acc, token| acc + token);
        write!(f, "export {}", s)
    }
}

impl Display for JsStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .tokens
            .iter()
            .fold(String::new(), |acc, token| acc + token);
        write!(f, "{}", s)
    }
}

pub trait ModuleImports {
    fn import_statement(&self, moduleFile: &str) -> String;
}

impl ModuleImports for Vec<ExportJsFunction> {
    fn import_statement(&self, moduleFile: &str) -> String {
        if !self.is_empty() {
            let imports =
                self.iter()
                    .fold(String::new(), |acc, js_function| match &js_function.name {
                        None => acc,
                        Some(name) => {
                            let sep = if acc.is_empty() { "" } else { ", " };
                            format!("{}{}{}", acc, sep, name)
                        }
                    });
            format!("import {{ {} }} from '{}';", imports, moduleFile)
        } else {
            "".to_string()
        }
    }
}

// TODO: This is a very basic way to achieve restoration of whitespace.  Maybe we look at an external prettifier?
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

pub fn extract_js_functions(source_text: &str) -> (Vec<JsStatement>, Vec<ExportJsFunction>) {
    let tokens = source_text.split_ascii_whitespace();
    let mut exports: Vec<ExportJsFunction> = Vec::new();
    let mut statements: Vec<JsStatement> = Vec::new();

    tokens.fold(State::Statement, |state, token| match state {
        State::Splitting => {
            if token == "function" {
                State::ExportJsFunction {
                    name: None,
                    tokens: Vec::from([restore_ws(token, 1)]),
                    depth: 0,
                    split: true,
                }
            } else {
                statements.push(JsStatement {
                    tokens: Vec::from([restore_ws(token, 0)]),
                });
                State::Statement
            }
        }
        State::SingleComment => {
            if token == "split" {
                State::Splitting
            } else {
                statements.push(JsStatement {
                    tokens: Vec::from([restore_ws(token, 0)]),
                });
                State::Statement
            }
        }
        State::Statement => {
            if token == "//" {
                statements.push(JsStatement {
                    tokens: Vec::new(),
                });
                State::SingleComment
            } else {
                // tokens.push(restore_ws(token, 0));
                statements.push(JsStatement {
                    tokens: Vec::from([restore_ws(token, 0)]),
                });
                State::Statement
            }
        }
        State::ExportJsFunction {
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
                State::ExportJsFunction {
                    name: name0,
                    tokens,
                    depth: depth + 1,
                    split,
                }
            } else if token == "}" {
                let depth0 = depth - 1;
                tokens.push(restore_ws(token, depth0));
                if depth0 > 0 {
                    State::ExportJsFunction {
                        name: name0,
                        tokens,
                        depth: depth0,
                        split,
                    }
                } else {
                    // if split {
                    //     exports.push(ExportJsFunction {
                    //         name: name0,
                    //         tokens,
                    //     });
                    // } else {
                    //     statements.push(JsStatement {
                    //         tokens,
                    //     });
                    // }
                    exports.push(ExportJsFunction {
                        name: name0,
                        tokens,
                    });

                    State::Statement
                }
            } else {
                tokens.push(restore_ws(token, depth));
                State::ExportJsFunction {
                    name: name0,
                    tokens,
                    depth,
                    split,
                }
            }
        }
    });

    (statements, exports)
}
