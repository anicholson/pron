use serde_json::Value;
use std::io::{self, BufRead};

struct Node {
    children: Vec<(String, Node)>,
    status: Option<String>,
}

impl Node {
    fn new() -> Node {
        Node { children: Vec::new(), status: None }
    }

    fn insert(&mut self, path: &[&str], status: &str) {
        if path.is_empty() {
            self.status = Some(status.to_string());
            return;
        }
        let idx = self
            .children
            .iter()
            .position(|(name, _)| name == path[0])
            .unwrap_or_else(|| {
                self.children.push((path[0].to_string(), Node::new()));
                self.children.len() - 1
            });
        self.children[idx].1.insert(&path[1..], status);
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
        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let event = match parsed.get("event").and_then(|v| v.as_str()) {
            Some(e) => e,
            None => continue,
        };
        if event == "started" || event == "suite" {
            continue;
        }
        let name = match parsed.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let name = name.replace('$', "::");
        let parts: Vec<&str> = name.split("::").collect();
        if parts.len() > 1 {
            root.insert(&parts[1..], event);
        } else {
            root.insert(&parts, event);
        }
    }

    root.print(0);
}