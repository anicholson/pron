use std::collections::BTreeMap;
use std::io::{self, BufRead};

fn extract_field(line: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\":\"");
    let start = line.find(&key)? + key.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

struct Node {
    children: BTreeMap<String, Node>,
    status: Option<String>,
}

impl Node {
    fn new() -> Node {
        Node { children: BTreeMap::new(), status: None }
    }

    fn insert(&mut self, path: &[&str], status: &str) {
        if path.is_empty() {
            self.status = Some(status.to_string());
            return;
        }
        let child = self.children
            .entry(path[0].to_string())
            .or_insert_with(Node::new);
        child.insert(&path[1..], status);
    }

    fn print(&self, indent: usize) {
        for (name, child) in &self.children {
            let prefix = "  ".repeat(indent);
            if child.children.is_empty() {
                let icon = match child.status.as_deref() {
                    Some("ok") => "\u{2713}",
                    Some("failed") => "\u{2717}",
                    _ => "?",
                };
                println!("{prefix}{icon} {name}");
            } else {
                println!("{prefix}{name}");
                child.print(indent + 1);
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut root = Node::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if !line.starts_with('{') {
            continue;
        }
        let event = match extract_field(&line, "event") {
            Some(e) => e,
            None => continue,
        };
        if event == "started" || event == "suite" {
            continue;
        }
        let name = match extract_field(&line, "name") {
            Some(n) => n,
            None => continue,
        };
        let name = name.replace('$', "::");
        let parts: Vec<&str> = name.split("::").collect();
        if parts.len() > 1 {
            root.insert(&parts[1..], &event);
        } else {
            root.insert(&parts, &event);
        }
    }

    root.print(0);
}
