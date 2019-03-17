use crate::error::Result;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn bool_is_false(b: &bool) -> bool {
    !*b
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    Home,
    Config,
    Absolute,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub path: PathBuf,
    pub location: Location,
    #[serde(skip_serializing_if = "bool_is_false", default)]
    pub recursive: bool,
    #[serde(skip_serializing_if = "bool_is_false", default)]
    pub symbolic: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub files: Vec<File>,
    pub git: Option<PathBuf>,
    pub dot: Option<PathBuf>,
}

impl Config {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = fs::File::open(path)?;
        serde_yaml::from_reader(&mut file).map_err(failure::Error::from)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = fs::File::create(path)?;
        serde_yaml::to_writer(&mut file, self).map_err(failure::Error::from)
    }

    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        ctx: &Context,
        path: P,
        recursive: bool,
    ) -> Result<()> {
        let path: &Path = path.as_ref();
        let abs = path_abs::PathAbs::new(path)?;
        let path = abs.as_path();
        let (path, loc) = ctx.clean_path(path);
        for file in &self.files {
            if file.path == path {
                println!("file {} is already added", path.display());
                return Ok(());
            }
        }
        self.files.retain(|f| !f.path.starts_with(&path));
        self.files.push(File {
            path,
            recursive,
            location: loc,
            symbolic: false,
        });
        Ok(())
    }

    pub fn set_git_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.git.replace(path.as_ref().to_owned());
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            files: Vec::new(),
            git: None,
            dot: None,
        }
    }
}

pub struct Context {
    pub home: PathBuf,
    pub xdg_config: PathBuf,
    // path to dot directory
    pub dot: PathBuf,
    pub dot_config: PathBuf,
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }

    pub fn clean_path<P: AsRef<Path>>(&self, path: P) -> (PathBuf, Location) {
        let path = path.as_ref();
        if let Ok(p) = path.strip_prefix("~") {
            return (p.to_owned(), Location::Home);
        }
        if let Ok(p) = path.strip_prefix(&self.xdg_config) {
            return (p.to_owned(), Location::Config);
        }
        if let Ok(p) = path.strip_prefix(&self.home) {
            return (p.to_owned(), Location::Home);
        }
        return (path.to_owned(), Location::Absolute);
    }

    pub fn get_path(&self, loc: &Location) -> PathBuf {
        match loc {
            Location::Home => self.home.to_owned(),
            Location::Config => self.xdg_config.to_owned(),
            Location::Absolute => PathBuf::from("/"),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let home = env::var("HOME").expect("HOME variable is not set.");
        let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));
        let dot = env::var("DOT_PATH").unwrap_or_else(|_| format!("{}/.dot", home));
        Context {
            home: PathBuf::from(home),
            xdg_config: PathBuf::from(config),
            dot: PathBuf::from(&dot),
            dot_config: PathBuf::from(&dot).join("config.yml"),
        }
    }
}
