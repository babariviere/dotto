use crate::error::{DotError, Result};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn bool_is_false(b: &bool) -> bool {
    !*b
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    Home,
    Config,
    Absolute,
}

// TODO: allow rename
// TODO: fix recursive not taken into account
// TODO: add exclude to hide secret files
// TODO: add whitelist and blacklist
#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub path: PathBuf,
    pub location: Location,
    #[serde(skip_serializing_if = "bool_is_false", default)]
    pub recursive: bool,
    #[serde(skip_serializing_if = "bool_is_false", default)]
    pub symbolic: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Git {
    pub path: PathBuf,
    pub location: Location,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub files: Vec<File>,
    git: Option<Git>,
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
        let (path, loc) = ctx.abs_clean_path(path)?;
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
            exclude: Vec::new(),
        });
        Ok(())
    }

    // TODO: clean this
    pub fn add_exclude(&mut self, ctx: &Context, exclude: &str) -> Result<()> {
        let exclude = exclude.trim();
        let _ = glob::Pattern::new(exclude).map_err(|e| DotError::wrap(exclude, e))?;
        let exclude_path = PathBuf::from(exclude);
        let mut components_left = Vec::new();
        let mut components_right = Vec::new();
        let mut left = true;
        for part in exclude_path.components() {
            use std::path::Component;
            match part {
                Component::Normal(s) => {
                    s.to_str().map(|s| {
                        if left && (s.contains("*") || s.contains("?") || s.contains("[")) {
                            left = false;
                        }
                        if left {
                            components_left.push(part);
                        } else {
                            components_right.push(part);
                        }
                    });
                }
                c => components_left.push(c),
            }
        }
        let root_path = components_left.into_iter().collect::<PathBuf>();
        let (mut path, loc) = ctx.abs_clean_path(root_path)?;
        for file in &mut self.files {
            if file.location != loc {
                continue;
            }
            if path.starts_with(&file.path) {
                if components_right.len() > 0 {
                    let trimmed_path = components_right.into_iter().collect::<PathBuf>();
                    path = path.join(trimmed_path);
                }
                let estr = path.strip_prefix(&file.path)?.display().to_string();
                if !file.exclude.contains(&estr) {
                    file.exclude.push(estr);
                }
                return Ok(());
            }
        }
        Err(DotError::NoMatch(exclude.to_string()).into())
    }

    pub fn git_dir(&self, ctx: &Context) -> Option<PathBuf> {
        self.git
            .as_ref()
            .map(|g| ctx.get_path(&g.location).join(&g.path))
    }

    pub fn set_git_dir<P: AsRef<Path>>(&mut self, ctx: &Context, path: P) {
        let (path, loc) = ctx.clean_path(path.as_ref());
        self.git.replace(Git {
            path,
            location: loc,
        });
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

    pub fn abs_clean_path<P: AsRef<Path>>(&self, path: P) -> Result<(PathBuf, Location)> {
        let path: &Path = path.as_ref();
        let abs = path_abs::PathAbs::new(path)
            .map_err(|e| DotError::wrap(path.display().to_string(), e))?;
        let path = abs.as_path();
        Ok(self.clean_path(path))
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
