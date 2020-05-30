use std::path::{Path, PathBuf};
use std::{fs, io};

use toml;

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Config is used to create a new store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The `path` field determines where the database will be created
    pub path: PathBuf,

    /// The `read_only` field specifies whether or not writes will be allowed
    #[serde(default)]
    pub read_only: bool,

    /// The `temporary` field specifies if the database will be destroyed on close
    #[serde(default)]
    pub temporary: bool,

    /// Enable compression by setting `use_compression` to true
    #[serde(default)]
    pub use_compression: bool,

    /// Specify the flush frequency
    #[serde(default)]
    pub flush_every_ms: Option<u64>,

    /// Specify the cache capacity
    #[serde(default)]
    pub cache_capacity: Option<u64>,
}

impl Config {
    /// Create a default configuration object
    pub fn new<P: AsRef<Path>>(p: P) -> Config {
        Config {
            path: p.as_ref().to_path_buf(),
            read_only: false,
            temporary: false,
            use_compression: false,
            flush_every_ms: None,
            cache_capacity: None,
        }
    }

    /// Save Config to an io::Write
    pub fn save_to<W: io::Write>(&self, mut w: W) -> Result<(), Error> {
        let s = match toml::to_string(self) {
            Ok(s) => s,
            Err(_) => return Err(Error::InvalidConfiguration),
        };
        w.write_all(s.as_ref())?;
        Ok(())
    }

    /// Save Config to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = fs::File::create(path.as_ref())?;
        self.save_to(file)
    }

    /// Load configuration from an io::Read
    pub fn load_from<R: io::Read>(mut r: R) -> Result<Config, Error> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        match toml::from_slice(buf.as_ref()) {
            Ok(cfg) => Ok(cfg),
            Err(_) => Err(Error::InvalidConfiguration),
        }
    }

    /// Load configuration to a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        let file = fs::File::open(path.as_ref())?;
        Self::load_from(file)
    }

    /// Set readonly field
    pub fn read_only(mut self, readonly: bool) -> Config {
        self.read_only = readonly;
        self
    }

    /// Set readonly field
    pub fn use_compression(mut self, use_compression: bool) -> Config {
        self.use_compression = use_compression;
        self
    }

    /// Toggle `temporary` value
    pub fn temporary(mut self, temporary: bool) -> Config {
        self.temporary = temporary;
        self
    }

    /// Set flush frequency
    pub fn flush_every_ms(mut self, ms: u64) -> Config {
        self.flush_every_ms = Some(ms);
        self
    }

    /// Set cache capacity
    pub fn cache_capacity(mut self, ms: u64) -> Config {
        self.cache_capacity = Some(ms);
        self
    }

    pub(crate) fn open(&mut self) -> Result<sled::Db, Error> {
        let config = sled::Config::new()
            .path(&self.path)
            .read_only(self.read_only)
            .temporary(self.temporary)
            .flush_every_ms(self.flush_every_ms)
            .use_compression(self.use_compression);
        let config = if let Some(cache_capacity) = self.cache_capacity {
            config.cache_capacity(cache_capacity)
        } else {
            config
        };
        let db = config.open()?;
        Ok(db)
    }
}
