use rand::{thread_rng, RngCore};
use sha2::Digest;

pub(crate) fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

pub(crate) fn digest(payload: &str) -> String {
    hex::encode(sha2::Sha256::digest(payload.as_bytes()))
}


#[derive(PartialEq, Debug, Clone)]
struct QueryNode {
    parent: Option<Box<QueryNode>>,
    path: String,
    children: Vec<QueryNode>,
}
pub(crate) fn parse_query(query: &str) -> Vec<String> {
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
    /*let mut queue: Vec<(String, Vec<&str>)> = vec![("".to_owned(), lines)];
    while let Some(c) = queue.pop() {}*/
    query_keys
}


#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn parse_test() {
        let query = "
            {
                personal{
                    name
                    sur
                }
            }
            ";
        let items = parse_query(query);
        eprintln!("{:?}", items);
    }
}
