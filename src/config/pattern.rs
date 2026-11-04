use globset::GlobBuilder;
use regex_automata::{meta::Regex, util::syntax};

// A bounded, compositional glob automaton. Alternatives share continuations;
// they are never expanded into a Cartesian list of whole patterns. The three
// tails keep recursive segments distinct from ordinary segment characters.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tail {
    Boundary,
    Component,
    Tree,
}
#[derive(Default)]
struct Node {
    epsilon: Vec<usize>,
    edges: Vec<(Test, usize)>,
}
enum Test {
    Literal(char),
    Scalar,
    Class(Regex),
}
pub(super) struct Matcher {
    nodes: Vec<Node>,
    accept: usize,
}

impl Matcher {
    pub(super) fn new(pattern: &str) -> Result<Self, ()> {
        if pattern.is_empty() || pattern.starts_with('!') {
            return Err(());
        }
        let mut compiler = Compiler {
            chars: pattern.chars().collect(),
            index: 0,
            nodes: vec![Node::default()],
        };
        let ends = compiler.sequence(vec![(0, Tail::Boundary)], false)?;
        let accept = compiler.node();
        for (node, tail) in ends {
            if tail == Tail::Tree {
                compiler.nodes[node].epsilon.push(accept);
            } else {
                compiler.nodes[node]
                    .edges
                    .push((Test::Literal('/'), accept));
            }
        }
        Ok(Self {
            nodes: compiler.nodes,
            accept,
        })
    }

    pub(super) fn is_match(&self, path: &str) -> bool {
        let mut active = self.closure(vec![0]);
        // A trailing separator gives ** the same zero-or-more complete-segment
        // semantics at the end and in the middle of a pattern.
        for ch in path.chars().chain(std::iter::once('/')) {
            let mut next = Vec::new();
            for node in active {
                for (test, target) in &self.nodes[node].edges {
                    let matches = match test {
                        Test::Literal(literal) => ch == *literal,
                        Test::Scalar => ch != '/',
                        Test::Class(regex) => regex.is_match(&*ch.encode_utf8(&mut [0; 4])),
                    };
                    if matches {
                        next.push(*target);
                    }
                }
            }
            active = self.closure(next);
            if active.is_empty() {
                return false;
            }
        }
        active.contains(&self.accept)
    }

    fn closure(&self, mut pending: Vec<usize>) -> Vec<usize> {
        let mut seen = vec![false; self.nodes.len()];
        let mut result = Vec::new();
        while let Some(node) = pending.pop() {
            if seen[node] {
                continue;
            }
            seen[node] = true;
            result.push(node);
            pending.extend(&self.nodes[node].epsilon);
        }
        result
    }
}

struct Compiler {
    chars: Vec<char>,
    index: usize,
    nodes: Vec<Node>,
}
impl Compiler {
    fn node(&mut self) -> usize {
        self.nodes.push(Node::default());
        self.nodes.len() - 1
    }
    fn merge(&mut self, ends: Vec<(usize, Tail)>) -> Vec<(usize, Tail)> {
        let mut result = Vec::new();
        for tail in [Tail::Boundary, Tail::Component, Tail::Tree] {
            let sources: Vec<_> = ends
                .iter()
                .filter(|(_, t)| *t == tail)
                .map(|(n, _)| *n)
                .collect();
            if sources.is_empty() {
                continue;
            }
            let target = self.node();
            for source in sources {
                self.nodes[source].epsilon.push(target);
            }
            result.push((target, tail));
        }
        result
    }
    fn sequence(
        &mut self,
        mut ends: Vec<(usize, Tail)>,
        in_brace: bool,
    ) -> Result<Vec<(usize, Tail)>, ()> {
        while let Some(&ch) = self.chars.get(self.index) {
            if in_brace && matches!(ch, ',' | '}') {
                break;
            }
            self.index += 1;
            match ch {
                '{' if !in_brace => {
                    let start = ends;
                    let mut branches = Vec::new();
                    loop {
                        branches.extend(self.sequence(start.clone(), true)?);
                        match self.chars.get(self.index) {
                            Some(',') => self.index += 1,
                            Some('}') => {
                                self.index += 1;
                                break;
                            }
                            _ => return Err(()),
                        }
                    }
                    ends = self.merge(branches);
                }
                '{' | '}' => return Err(()),
                '/' => {
                    let target = self.node();
                    for (node, tail) in ends {
                        if tail == Tail::Tree {
                            self.nodes[node].epsilon.push(target);
                        } else {
                            self.nodes[node].edges.push((Test::Literal('/'), target));
                        }
                    }
                    ends = vec![(target, Tail::Boundary)];
                }
                '*' if self.chars.get(self.index) == Some(&'*') => {
                    self.index += 1;
                    if ends.iter().any(|(_, tail)| *tail != Tail::Boundary) {
                        return Err(());
                    }
                    let start = self.node();
                    let component = self.node();
                    for (node, _) in ends {
                        self.nodes[node].epsilon.push(start);
                    }
                    self.nodes[start].edges.push((Test::Scalar, component));
                    self.nodes[component].edges.push((Test::Scalar, component));
                    self.nodes[component]
                        .edges
                        .push((Test::Literal('/'), start));
                    ends = vec![(start, Tail::Tree)];
                }
                ch => {
                    if ends.iter().any(|(_, tail)| *tail == Tail::Tree) {
                        return Err(());
                    }
                    let target = self.node();
                    let test = match ch {
                        '?' | '*' => Test::Scalar,
                        '[' => {
                            let start = self.index - 1;
                            let first = self.index;
                            loop {
                                let c = *self.chars.get(self.index).ok_or(())?;
                                if c == '/' {
                                    return Err(());
                                }
                                let initial = self.index == first
                                    || (self.index == first + 1
                                        && matches!(self.chars[first], '!' | '^'));
                                self.index += 1;
                                if c == ']' && !initial {
                                    break;
                                }
                            }
                            let class: String = self.chars[start..self.index].iter().collect();
                            Test::Class(class_regex(&class)?)
                        }
                        _ => Test::Literal(ch),
                    };
                    let join = self.node();
                    for (node, _) in ends {
                        self.nodes[node].epsilon.push(join);
                    }
                    if ch == '*' {
                        self.nodes[join].epsilon.push(target);
                        self.nodes[join].edges.push((test, join));
                    } else {
                        self.nodes[join].edges.push((test, target));
                    }
                    ends = vec![(target, Tail::Component)];
                }
            }
        }
        Ok(ends)
    }
}

fn class_regex(pattern: &str) -> Result<Regex, ()> {
    let glob = GlobBuilder::new(pattern)
        .literal_separator(true)
        .backslash_escape(false)
        .case_insensitive(false)
        .build()
        .map_err(|_| ())?;
    let expression = segment_classes(&unicode_expression(glob.regex())?)?;
    Regex::builder()
        .syntax(
            syntax::Config::new()
                .unicode(true)
                .utf8(true)
                .dot_matches_new_line(true),
        )
        .build(&expression)
        .map_err(|_| ())
}

// The glob compiler's negated classes may include '/'. The contract treats
// classes as segment characters, so use the real Unicode engine's set
// intersection rather than allowing an override to cross a separator.
fn segment_classes(input: &str) -> Result<String, ()> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            result.push(ch);
            result.push(chars.next().ok_or(())?);
        } else if ch == '[' {
            result.push_str("[[");
            loop {
                let ch = chars.next().ok_or(())?;
                result.push(ch);
                if ch == '\\' {
                    result.push(chars.next().ok_or(())?);
                } else if ch == ']' {
                    break;
                }
            }
            result.push_str("&&[^/]]");
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}

fn unicode_expression(input: &str) -> Result<String, ()> {
    let input = input.strip_prefix("(?-u)").ok_or(())?;
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' && bytes.get(index + 1) == Some(&b'\\') {
            result.push_str("\\\\");
            index += 2;
            continue;
        }
        let mut utf8 = Vec::new();
        while index + 3 < bytes.len() && &bytes[index..index + 2] == b"\\x" {
            let value = u8::from_str_radix(&input[index + 2..index + 4], 16).map_err(|_| ())?;
            if value < 128 {
                break;
            }
            utf8.push(value);
            index += 4;
        }
        if !utf8.is_empty() {
            result.push_str(std::str::from_utf8(&utf8).map_err(|_| ())?);
        } else {
            let ch = input[index..].chars().next().ok_or(())?;
            result.push(ch);
            index += ch.len_utf8();
        }
    }
    Ok(result)
}
