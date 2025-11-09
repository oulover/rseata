use std::path::Path;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
struct Package {
  path: PathBuf,
  _name: String,
  _components: Vec<String>,
}

impl Package {
  fn from_path(base: &Path, path: PathBuf) -> Package {
    let components: Vec<String> = path
        .strip_prefix(base)
        .unwrap()
        .components()
        .filter_map(|component| match component {
          std::path::Component::Normal(path) => path.to_str().map(ToOwned::to_owned),
          _ => None,
        })
        .collect();
    Package {
      path,
      _name: components
          .iter()
          .last()
          .and_then(|v| Path::new(v).file_stem())
          .and_then(|v| v.to_str().map(ToOwned::to_owned))
          .unwrap(),
      _components: components,
    }
  }
}

fn main() {
  let base = Path::new("./proto");

  let dir = WalkDir::new("./proto");
  let entries = dir
      .into_iter()
      .filter_entry(|e| !is_hidden(e))
      .collect::<Result<Vec<_>, _>>()
      .unwrap();
  let packages: Vec<_> = entries
      .into_iter()
      .filter_map(|entry| {
        if entry.file_type().is_file() {
          Some(Package::from_path(base, entry.into_path()))
        } else {
          None
        }
      })
      .collect();
  let mut builder = tonic_prost_build::configure();

  builder = builder.build_client(false).build_server(false);

  #[cfg(feature = "client")]
  {
    builder = builder.build_client(true);
  }

  #[cfg(feature = "server")]
  {
    builder = builder.build_server(true);
  }

  let paths: Vec<&Path> = packages.iter().map(|p| p.path.as_ref()).collect();

  builder.compile_protos(paths.as_ref(),&[Path::new("./proto")]).unwrap();
}

fn is_hidden(entry: &DirEntry) -> bool {
  entry
      .file_name()
      .to_str()
      .map(|s| s.starts_with("."))
      .unwrap_or(false)
}
