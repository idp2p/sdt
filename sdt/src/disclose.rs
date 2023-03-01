use crate::{error::SdtError, node::SdtNode, SdtNodeKind, SdtResult};

#[derive(PartialEq, Debug, Clone)]
struct QueryNode {
    parent: Option<Box<QueryNode>>,
    path: String,
    children: Vec<QueryNode>,
}

fn parse_query(query: &str) -> Vec<String> {
    let mut query_keys: Vec<String> = vec![];
    let lines: Vec<&str> = query.trim().split("\n").map(|x| x.trim()).collect();
    let mut node = QueryNode {
        parent: None,
        path: "".to_string(),
        children: vec![],
    };
    for line in lines {
        if line.ends_with("{") {
            let new_node = QueryNode {
                path: format!("{}{}/", node.path, line.replace("{", "")),
                parent: Some(Box::new(node.clone())),
                children: vec![],
            };
            node.children.push(new_node.clone());
            node = new_node;
        } else if line.ends_with("}") {
            node = *node.parent.unwrap();
        } else {
            query_keys.push(format!("{}{}/", node.path, line));
        }
    }
    query_keys
}

pub fn disclose(node: &SdtNode, query: &str) -> Result<SdtResult, SdtError> {
    let query_keys = parse_query(query);
    let mut result = node.try_into()?;
    let mut queue: Vec<(String, &mut SdtResult)> = vec![("".to_owned(), &mut result)];
    while let Some((path, cn)) = queue.pop() {
        let path_key = format!("{}{}/", path, cn.key);
        if !query_keys.contains(&path_key) {
            let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
            if matched {
                if let Some(v) = &mut cn.value {
                    match v {
                        SdtNodeKind::Branch { children } => {
                            for n in children {
                                queue.push((path_key.clone(), n));
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                cn.value = None;
            }
        }
    }
    Ok(result)
}
