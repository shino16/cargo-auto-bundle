use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

pub type ModPath = Vec<String>;

mod compile;
mod traverse;

#[derive(StructOpt)]
#[structopt(bin_name("cargo"))]
enum Opt {
    AutoBundle {
        #[structopt(long = "crate", default_value = ".")]
        crate_path: PathBuf,
        #[structopt(long, default_value = "src/main.rs")]
        entry_point: PathBuf,
        #[structopt(short, long)]
        list_deps: bool,
    },
}

fn main() -> Result<()> {
    let Opt::AutoBundle {
        crate_path,
        entry_point,
        list_deps,
    } = Opt::from_args();

    let crate_path = crate_path.canonicalize()?;

    let toml_path = crate_path.join("Cargo.toml").canonicalize()?;
    let manifest = cargo_toml::Manifest::from_path(&toml_path)?;
    let crate_name = if let Some(lib) = manifest.lib {
        lib.name.unwrap()
    } else {
        panic!("No lib package found.");
    };

    let crate_root = crate_path.join("src").canonicalize()?;

    let (paths, file_paths, mods_visibility) =
        traverse::Traverse::new(&crate_root, &crate_name, &entry_point)?.run()?;

    if list_deps {
        for file_path in file_paths {
            println!("{}", file_path.to_string_lossy());
        }
        return Ok(());
    }

    let mut result = compile::compile_entry(&entry_point, &crate_name)?;
    result += "\n\n\n";
    result += &compile::compile(&crate_name, &paths, &file_paths, mods_visibility)?;

    println!("{}", result);

    Ok(())
}
