use std::path::Path;
use std::process::Command;
use std::str;

#[derive(Debug)]
pub enum RegistryError {
    NoSuchPath,
    NotADirectory,
    ModuleError(ModuleError),
    IOError(std::io::Error),
}

impl From<std::io::Error> for RegistryError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<ModuleError> for RegistryError {
    fn from(err: ModuleError) -> Self {
        Self::ModuleError(err)
    }
}

/// A registry of all available modules to osbuild.
pub struct Registry<'a> {
    modules: Vec<Module<'a>>,
}

impl Registry<'_> {
    /// Create a new registry
    pub fn new<'a>(modules: Vec<Module<'a>>) -> Registry<'a> {
        Registry { modules }
    }

    /// Create a new empty registry
    pub fn new_empty() -> Self {
        Self { modules: vec![] }
    }

    /// Add the 'well-known' locations where `osbuild` modules might be located.
    /// XXX: decide if we actually want this or if we always want to be explicit and only load data
    /// from explicitly provided paths in the binaries.
    pub fn add_well_known(&mut self) -> Result<(), RegistryError> {
        Ok(())
    }

    /// Find a module by its name.
    pub fn by_name(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|&module| module.name == name)
    }

    /// Find modules by their kind.
    pub fn by_kind(&self, kind: Kind) -> Option<Vec<&Module>> {
        let modules: Vec<&Module> = self
            .modules
            .iter()
            .filter(|&module| module.kind == kind)
            .collect();

        (!modules.is_empty()).then_some(modules)
    }
}

/// Kind of a module.
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Kind {
    Stage,
    Assembler,
    Source,
    Runner,
    Mount,
    Device,
    Input,
}

// The default paths where certain modules are located on a default install, note that
// compatibility should be checked on these XXX
const WELL_KNOWN_MODULE_PATH_ASSEMBLER: &'static str = "/usr/lib/osbuild/assemblers";
const WELL_KNOWN_MODULE_PATH_DEVICE: &'static str = "/usr/lib/osbuild/devices";
const WELL_KNOWN_MODULE_PATH_INPUT: &'static str = "/usr/lib/osbuild/inputs";
const WELL_KNOWN_MODULE_PATH_MOUNT: &'static str = "/usr/lib/osbuild/mounts";
const WELL_KNOWN_MODULE_PATH_RUNNER: &'static str = "/usr/lib/osbuild/runners";
const WELL_KNOWN_MODULE_PATH_SOURCE: &'static str = "/usr/lib/osbuild/sources";
const WELL_KNOWN_MODULE_PATH_STAGE: &'static str = "/usr/lib/osbuild/stages";

/// Errors that happen during execution of a module.
#[derive(Debug)]
pub enum ModuleError {
    /// Tried to create a module with an unparseable path.
    CantGetFilename,

    /// Tried to create a module with a non-existing path.
    NoSuchPath,

    IOError(std::io::Error),

    /// The output of the module was not decodable as UTF-8.
    Utf8Error(std::str::Utf8Error),
}

impl From<std::io::Error> for ModuleError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<std::str::Utf8Error> for ModuleError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

/// A module.
pub struct Module<'a> {
    /// The type of the module.
    kind: Kind,

    /// The path of the module
    path: &'a str,

    /// The name of the module, the filename part of the path.
    name: &'a str,

    /// The schema of the module, this is initially `None` but once requested by `get_schema` the
    /// result will be cached in this field for faster retrieval.
    schema: Option<String>,
}

impl Module<'_> {
    fn new<'a>(kind: Kind, path: &'a str) -> Result<Module<'a>, ModuleError> {
        let p = Path::new(path);

        if !p.exists() {
            Err(ModuleError::NoSuchPath)
        } else {
            let f = p.file_name().ok_or(ModuleError::CantGetFilename)?;

            Ok(Module {
                kind,
                path,
                name: f.to_str().unwrap(),
                schema: None,
            })
        }
    }

    /// Get the schema for this module by executing the module with the `--schema` argument,
    /// results are cached.
    fn get_schema(&self) -> Result<String, ModuleError> {
        match self.schema.as_ref() {
            Some(schema) => Ok(schema.to_string()),
            None => {
                let command = Command::new(self.path).args(["--schema"]).output()?;
                let output = str::from_utf8(&command.stdout)?.to_string();

                Ok(output)
            }
        }
    }
}

#[cfg(test)]
mod test;
