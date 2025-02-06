use markup5ever_rcdom::{Handle, NodeData};
use crate::javascript::{ExportJsFunction, JsStatement, ModuleImports};

pub fn extract_javascript(handle: &Handle) -> Vec<String> {
    let node = handle;
    let mut scripts: Vec<String> = Vec::new();

    extract_javascript_rec(node, &mut scripts);
    scripts
}

fn extract_javascript_rec(handle: &Handle, scripts: &mut Vec<String>) {
    let node = handle;

    match node.data {
        NodeData::Element { ref name, .. } if name.local.to_lowercase() == "script" => {
            node.children.borrow().iter().for_each(|child| {
                if let NodeData::Text { ref contents } = child.data {
                    let text = contents.borrow().to_string();
                    if !text.is_empty() {
                        scripts.push(text);
                    }
                }
            })
        }
        NodeData::ProcessingInstruction { .. } => unreachable!(),
        _ => {}
    }

    for child in node.children.borrow().iter() {
        extract_javascript_rec(child, scripts);
    }
}


pub fn update_script_tag<M: ModuleImports>(handle: &Handle, js_statements: &Vec<JsStatement>, imports: &M, module_file: &str) {
    let node = handle;
    match node.data {
        NodeData::Element { ref name, .. } if name.local.to_lowercase() == "script" => {
            node.children.borrow().iter().for_each(|child| {
                if let NodeData::Text { ref contents } = child.data {

                    println!("Number of statements: {}", js_statements.len());

                   let rendered_statements = js_statements.iter().fold(String::new(), |acc, js_statement| {
                        let text: String = js_statement.to_owned().into();
                        format!("{} {}", acc, text)
                    });

                    let mut contents = contents.borrow_mut();
                    let all = format!("\n{}\n{}", imports.import_statement(module_file), rendered_statements);
                    *contents = all.into()
                }
            })
        }
        _ => {}
    }

    for child in node.children.borrow().iter() {
        update_script_tag(child, js_statements, imports, module_file);
    }
}
