use lazy_static::lazy_static;
use ropey::Rope;
use std::mem;
use tree_sitter::{InputEdit, Node, Parser, Point, Query, QueryCursor, Range, Tree};

#[derive(Debug)]
pub struct Span {
    pub kind: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct Item {
    pub kind: String,
    pub range: Range,
}

pub struct Editor {
    rope: Rope,
    parser: Parser,
    tree: Tree,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(text, None).unwrap();

        Self {
            rope: Rope::from_str(text),
            parser,
            tree,
        }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn insert(&mut self, line: usize, col: usize, text: &str) -> Tree {
        let idx = self.rope.line_to_char(line) + col;
        self.rope.insert(idx, text);

        let byte_idx = self.rope.char_to_byte(idx);
        let edit = InputEdit {
            start_byte: byte_idx,
            old_end_byte: byte_idx,
            new_end_byte: byte_idx + text.len(),
            start_position: Point::new(line, col),
            old_end_position: Point::new(line, col),
            new_end_position: Point::new(line, col + text.len()),
        };
        self.tree.edit(&edit);
        let tree = self
            .parser
            .parse_with(
                &mut |idx, _| self.rope.chunks_at_byte(idx).0.next().unwrap_or_default(),
                Some(&self.tree),
            )
            .unwrap();

        mem::replace(&mut self.tree, tree)
    }

    pub fn highlights(&self) -> Vec<Item> {
        lazy_static! {
            static ref QUERY: Query = Query::new(
                tree_sitter_rust::language(),
                r#"
                    ["fn" "for" "while"] @keyword
                "#,
            )
            .unwrap();
        }

        let mut query_cursor = QueryCursor::new();
        let rope = &self.rope;
        let matches = query_cursor.matches(&QUERY, self.tree.root_node(), move |node: Node| {
            rope.byte_slice(node.start_byte()..node.end_byte())
                .chunks()
                .map(move |chunk| chunk.as_bytes())
        });
        matches
            .flat_map(|mat| {
                mat.captures.iter().map(|capture| {
                    let range = capture.node.range();
                    let kind = capture.node.kind().to_owned();
                    Item { range, kind }
                })
            })
            .collect()
    }

    pub fn spans(&self) -> Vec<Span> {
        let highlights = self.highlights();
        let mut iter = self.rope.bytes().enumerate().peekable();
        let mut spans = Vec::new();
        let mut start = 0;

        while let Some((idx, _c)) = iter.next() {
            for highlight in highlights.iter() {
                if highlight.range.start_byte <= idx && highlight.range.end_byte >= idx {
                    if start < idx {
                        spans.push(Span {
                            kind: None,
                            text: self.rope.slice(start..idx).to_string(),
                        })
                    }

                    let mut end = idx;
                    'a: while let Some((next_idx, _)) = iter.peek() {
                        if highlight.range.start_byte <= *next_idx
                            && highlight.range.end_byte >= *next_idx
                        {
                            iter.next();
                        } else {
                            end = *next_idx;
                            break 'a;
                        }
                    }
                    spans.push(Span {
                        kind: Some(highlight.kind.clone()),
                        text: self.rope.slice(idx..end).to_string(),
                    });
                    start = end;
                }
            }
        }

        spans.push(Span {
            kind: None,
            text: self.rope.slice(start..).to_string(),
        });

        spans
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    #[test]
    fn it_works() {
        let editor = Editor::new("fn main() { for i in 0..2 {} }");
        dbg!(editor.spans());
    }
}
