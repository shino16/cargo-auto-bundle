use anyhow::{Error, Result};
use itertools::Itertools as _;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use syn::visit::Visit;

use super::ModPath;

pub struct Traverse {
    crate_root: PathBuf,
    crate_name: String,
    todo: Vec<ModPath>,
    exclude: PathBuf,
    mods_location: BTreeMap<ModPath, (PathBuf, ModPath)>,
    mods_visibility: BTreeMap<ModPath, String>,
}

impl Traverse {
    pub fn new(crate_root: &Path, crate_name: &str, entry_point: &Path) -> Result<Self> {
        let use_paths = visit_use_file(&entry_point.canonicalize()?)?;

        Ok(Traverse {
            crate_root: crate_root.to_owned(),
            crate_name: crate_name.to_owned(),
            todo: use_paths
                .into_iter()
                .filter(|path| path[0] == crate_name)
                .collect(),
            exclude: crate_root.join("bin"),
            mods_location: BTreeMap::new(),
            mods_visibility: BTreeMap::new(),
        })
    }

    fn canonicalize(&self, mut path: ModPath) -> ModPath {
        // TODO: convert relative path to absolute path
        if path[0] == "crate" {
            path[0] = self.crate_name.clone();
        }
        path
    }

    fn find_mod_file(&self, mod_path: &ModPath) -> Result<&(PathBuf, ModPath)> {
        let mut i = mod_path.len();
        while !self.mods_location.contains_key(&mod_path[..i]) {
            i -= 1;
        }
        Ok(self.mods_location.get(&mod_path[..i]).unwrap())
    }

    fn visit_use(&self, path: &ModPath) -> Result<Vec<ModPath>> {
        let paths = visit_use_file(&self.find_mod_file(&path)?.0)?;

        let canonical: Result<Vec<_>, _> = paths
            .into_iter()
            .map(|p| self.canonicalize(p))
            .filter(|path| path[0] == self.crate_name)
            .map(|p| self.find_mod_file(&p).map(|(_, p)| p.to_owned()))
            .collect();

        Ok(canonical?.into_iter().unique().collect())
    }

    fn search_mods(&mut self, file_path: &mut PathBuf, path: &mut ModPath) -> Result<()> {
        struct Visitor {
            mods: Vec<ModPath>,
            path: ModPath,
        }
        impl<'ast> Visit<'ast> for Visitor {
            fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
                self.path.push(item.ident.to_string());
                if item.content.is_some() {
                    self.mods.push(self.path.clone());
                }
                syn::visit::visit_item_mod(self, item);
                self.path.pop();
            }
        }

        for entry in std::fs::read_dir(&file_path)? {
            let entry = entry?;
            if entry.path() == self.exclude {
                continue;
            }
            let name = entry.file_name();
            file_path.push(name.clone());
            let name_string = name
                .into_string()
                .map_err(|_| Error::msg(format!("Cannot open {:?}", file_path)))?;
            if entry.metadata()?.is_dir() {
                path.push(name_string);
                self.search_mods(file_path, path)?;
                path.pop();
            } else {
                let name_str = &name_string[..name_string.len() - 3];
                if name_str != "mod" && name_str != "lib" {
                    path.push(name_str.to_owned());
                }
                self.mods_location
                    .insert(path.to_owned(), (file_path.to_owned(), path.to_owned()));
                let content = std::fs::read_to_string(&file_path)?;
                let file = syn::parse_file(&content)?;
                let mut visitor = Visitor {
                    mods: Vec::new(),
                    path: path.to_owned(),
                };
                visitor.visit_file(&file);
                for mod_path in visitor.mods {
                    self.mods_location
                        .insert(mod_path, (file_path.clone(), path.clone())); // TODO
                }
                if name_str != "mod" && name_str != "lib" {
                    path.pop();
                }
            }
            file_path.pop();
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(Vec<ModPath>, Vec<PathBuf>, BTreeMap<ModPath, String>)> {
        self.search_mods(
            &mut self.crate_root.clone(),
            &mut vec![self.crate_name.clone()],
        )?;

        let mut result = Vec::new();
        self.todo = self
            .todo
            .iter()
            .map(|path| self.find_mod_file(path).map(|(_, p)| p.to_owned()))
            .try_collect()?;
        let mut pushed = self.todo.clone();
        while let Some(path) = self.todo.pop() {
            result.push(path.clone());
            let uses = self.visit_use(&path)?;
            for path in uses {
                if !pushed.contains(&path) {
                    self.todo.push(path.clone());
                    pushed.push(path);
                }
            }
        }
        result.sort();
        result.dedup();
        let paths = result
            .iter()
            .map(|path| self.find_mod_file(path).map(|(p, _)| p.to_owned()))
            .try_collect()?;
        Ok((result, paths, std::mem::take(&mut self.mods_visibility)))
    }
}

fn visit_use_file(path: &Path) -> Result<Vec<ModPath>> {
    use syn::UseTree::{self, *};
    fn dfs(tree: &UseTree, prefix: &mut ModPath, buf: &mut Vec<ModPath>) {
        match tree {
            Path(path) => {
                prefix.push(path.ident.to_string());
                dfs(&*path.tree, prefix, buf);
                prefix.pop().unwrap();
            }
            Name(name) => {
                prefix.push(name.ident.to_string());
                buf.push(prefix.clone());
                prefix.pop();
            }
            Rename(rename) => {
                prefix.push(rename.ident.to_string());
                buf.push(prefix.clone());
                prefix.pop();
            }
            Glob(_) => {
                buf.push(prefix.clone());
            }
            Group(group) => {
                group.items.iter().for_each(|tree| dfs(tree, prefix, buf));
            }
        }
    }

    #[derive(Default)]
    struct Visitor {
        paths: Vec<ModPath>,
    }
    impl<'ast> Visit<'ast> for Visitor {
        fn visit_use_tree(&mut self, item: &'ast syn::UseTree) {
            dfs(item, &mut Vec::new(), &mut self.paths);
        }
    }

    let content = std::fs::read_to_string(path)?;
    let mut visitor = Visitor::default();
    visitor.visit_file(&syn::parse_file(&content)?);

    Ok(visitor.paths)
}
