use super::ModPath;
use anyhow::Result;
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn compile(
    crate_name: &str,
    paths: &[ModPath],
    file_paths: &[PathBuf],
    mod_visibility: BTreeMap<ModPath, String>,
) -> Result<String> {
    let mut res = String::new();
    let mut location = ModPath::new();
    for (path, file_path) in paths.into_iter().zip(file_paths) {
        let base = location
            .iter()
            .zip(path.iter())
            .take_while(|(a, b)| a == b)
            .count();
        while location.len() > base {
            let p = location.pop().unwrap();
            res += &format!("\n}}  // mod {}\n", p);
        }
        while location.len() < path.len() {
            let name = &path[location.len()];
            location.push(name.clone());
            if mod_visibility
                .get(&location)
                .filter(|s| s.is_empty())
                .is_some()
            {
                res += &format!("\nmod {} {{\n", name);
            } else {
                let vis = mod_visibility
                    .get(&location)
                    .cloned()
                    .unwrap_or("pub".to_owned());
                res += &format!("\n{} mod {} {{\n", vis, name);
            }
        }
        res += "\n";
        res += &read_proc(&file_path, crate_name, false)?;
    }
    while let Some(p) = location.pop() {
        res += &format!("\n}}  // mod {}\n", p);
    }
    Ok(res)
}

pub fn compile_entry(path: &Path, crate_name: &str) -> Result<String> {
    Ok(read_proc(path, crate_name, true)?)
}

fn read_proc(file_path: &Path, crate_name: &str, external: bool) -> Result<String> {
    use syn::visit::Visit;
    #[derive(Default)]
    struct Visitor<'ast> {
        use_spans: Vec<(&'ast Ident, Span)>,
        mod_spans: Vec<(Span, Span)>,
    }
    impl<'ast> Visit<'ast> for Visitor<'ast> {
        fn visit_use_tree(&mut self, item: &'ast syn::UseTree) {
            let ident = match item {
                syn::UseTree::Path(path) => &path.ident,
                syn::UseTree::Name(name) => &name.ident,
                _ => return,
            };
            self.use_spans.push((&ident, ident.span()));
        }
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            use quote::ToTokens as _;
            if item.semi.is_some() {
                let mut iter = item.to_token_stream().into_iter();
                let start = iter.next().unwrap().span();
                let end = iter.last().unwrap().span();
                self.mod_spans.push((start, end));
            }
            syn::visit::visit_item_mod(self, item);
        }
    }

    let content = std::fs::read_to_string(file_path)?;
    let file = syn::parse_file(&content)?;
    let mut visitor = Visitor::default();
    visitor.visit_file(&file);

    let mut targets = Vec::new();
    for (ident, span) in visitor.use_spans {
        if !external && ident.to_string() == "crate" {
            targets.push((span.end(), span.end(), format!("::{}", crate_name)));
        }
        if external && ident.to_string() == crate_name {
            targets.push((span.start(), span.start(), "crate::".to_owned()));
        }
    }
    for (start, end) in visitor.mod_spans {
        targets.push((start.start(), end.end(), "".to_owned()));
    }

    targets.sort();

    let lines = content.lines().collect_vec();
    if lines.is_empty() {
        return Ok("".to_owned());
    }

    let (mut line_pos, mut col_pos) = (0, 0);
    let mut res = String::new();

    for (start, end, pat) in targets {
        while line_pos < start.line - 1 {
            res += &lines[line_pos][col_pos..];
            res += "\n";
            line_pos += 1;
            col_pos = 0;
        }
        if pat.is_empty()
            && lines[start.line - 1][..start.column]
                .chars()
                .all(|c| c.is_ascii_whitespace())
            && lines[end.line - 1][end.column..]
                .chars()
                .all(|c| c.is_ascii_whitespace())
        {
            line_pos = end.line;
            col_pos = 0;
        } else {
            res += &lines[line_pos][..start.column];
            res += &pat;
            line_pos = end.line - 1;
            col_pos = end.column;
        }
    }

    if line_pos < lines.len() {
        res += &lines[line_pos][col_pos..];
        res += "\n";
        lines[line_pos + 1..].into_iter().for_each(|line| {
            res += line;
            res += "\n";
        });
    }

    Ok(res)
}
