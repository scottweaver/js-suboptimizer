use crate::error::Error;
use crate::prelude::*;
use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use thiserror::__private::AsDisplay;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Manifest {
    pub module_name: String,
    pub markup_name: String,
    pub package_name: String,
}

impl Manifest {
    pub fn from_markup_file(file_name: &PathBuf, package_name: &Option<String>) -> Result<Manifest> {
        let simple_name = file_name.file_stem().ok_or_else(|| {
            Error::Manifest(format!(
                "Could not extract file name strem from '{}'",
                &file_name.as_display()
            ))
        })?;
        let simple_name = simple_name.to_str().ok_or_else(|| {
            Error::Manifest("Could not convert 'OsStr' file name to string".to_string())
        })?;
        
        let package_name = package_name.as_ref().map(|s| s.as_str()).unwrap_or(simple_name);

        Ok(Manifest {
            module_name: format!("{}-module.js", simple_name),
            markup_name: format!("{}.html", simple_name),
            package_name: package_name.to_string(),
        })
    }

    pub fn new(module_name: String, markup_name: String, package_name: String) -> Self {
        Self {
            module_name,
            markup_name,
            package_name
        }
    }

    pub fn manifest_files(&self) -> Result<(File, File)> {
        let markup_name = &self.markup_name;
        let target_dir = &PathBuf::from(&self.package_name);

        create_dir_all(target_dir)?;
        let module_path = target_dir.join(&self.module_name);
        let markup_path = target_dir.join(&self.markup_name);
        let mut markup_file = File::create(markup_path)?;
        let mut module_file = File::create(module_path)?;
        Ok((markup_file, module_file))
    }
}
