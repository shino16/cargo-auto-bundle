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
    macros: &[String],
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
        res += &read_process(&file_path, crate_name, false, macros)?;
    }
    while let Some(p) = location.pop() {
        res += &format!("\n}}  // mod {}\n", p);
    }
    Ok(reduce_newline(res))
}

fn reduce_newline(mut s: String) -> String {
    let bytes = unsafe { s.as_bytes_mut() };
    let mut j = 0;
    let mut newline_cnt = 0;
    for i in 0..bytes.len() {
        if bytes[i] == b'\n' {
            newline_cnt += 1;
        } else {
            newline_cnt = 0;
        }
        if newline_cnt <= 2 {
            bytes[j] = bytes[i];
            j += 1;
        }
    }
    s.truncate(j);
    s
}

pub fn compile_entry(path: &Path, crate_name: &str, macros: &[String]) -> Result<String> {
    Ok(read_process(path, crate_name, true, macros)?)
}

fn read_process<'a>(
    file_path: &Path,
    crate_name: &'a str,
    external: bool,
    macros: &[String],
) -> Result<String> {
    use syn::visit::Visit;
    struct Visitor<'ast, 'a, 'b> {
        use_spans: Vec<(&'ast Ident, Span)>,
        mod_spans: Vec<(Span, Span)>,
        crate_name: &'a str,
        macros: &'b [String],
    }
    impl<'ast, 'a, 'b> Visit<'ast> for Visitor<'ast, 'a, 'b> {
        fn visit_use_tree(&mut self, item: &'ast syn::UseTree) {
            let ident = match item {
                syn::UseTree::Path(path) => {
                    if let syn::UseTree::Name(ref name) = *path.tree {
                        if path.ident == self.crate_name
                            && self.macros.contains(&name.ident.to_string())
                        {
                            return;
                        }
                    }
                    &path.ident
                }
                syn::UseTree::Name(name) => &name.ident,
                _ => return,
            };
            self.use_spans.push((&ident, ident.span()));
        }
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            use quote::ToTokens;
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
    let mut visitor = Visitor {
        use_spans: Vec::new(),
        mod_spans: Vec::new(),
        crate_name: if external { crate_name } else { "crate" },
        macros,
    };
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
