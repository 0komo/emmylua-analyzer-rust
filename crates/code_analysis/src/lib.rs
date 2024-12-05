mod compilation;
mod config;
mod db_index;
mod diagnostic;
mod semantic;
mod vfs;

use std::{env, path::PathBuf, sync::Arc};

#[allow(unused)]
pub use compilation::*;
pub use config::{Emmyrc, load_configs};
#[allow(unused)]
pub use diagnostic::*;
use log::{error, info};
use lsp_types::Uri;
#[allow(unused)]
pub use vfs::*;

#[derive(Debug)]
pub struct EmmyLuaAnalysis {
    pub compilation: LuaCompilation,
    pub emmyrc: Arc<Emmyrc>,
}

impl EmmyLuaAnalysis {
    pub fn new() -> Self {
        let emmyrc = Arc::new(Emmyrc::default());
        Self {
            compilation: LuaCompilation::new(emmyrc.clone()),
            emmyrc
        }
    }

    pub fn init_std_lib(&mut self) -> Option<()> {
        let resource_dir = self.get_resource_dir();
        match resource_dir {
            Some(resource_dir) => {
                info!("resource dir: {:?}, loading ...", resource_dir);
                let std_lib_dir = resource_dir.join("std");
                self.add_workspace_root(std_lib_dir.clone());
                let match_pattern = vec!["**/*.lua".to_string()];
                let files =
                    load_workspace_files(&std_lib_dir, &match_pattern, &Vec::new(), None).ok()?;

                let files = files.into_iter().map(|file| file.into_tuple()).collect();
                self.update_files_by_path(files);
            }
            None => {
                error!("Failed to find resource directory, std lib will not be loaded.");
            }
        }

        Some(())
    }

    pub fn get_resource_dir(&self) -> Option<PathBuf> {
        let exe_path = env::current_exe().ok()?;
        let mut current_dir = exe_path.parent()?.to_path_buf();

        loop {
            let potential = current_dir.join("resources");
            info!("try location resource dir: {:?} ...", potential);
            if potential.is_dir() {
                return Some(potential);
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break,
            }
        }

        None
    }

    pub fn get_file_id(&self, uri: &Uri) -> Option<FileId> {
        self.compilation.get_db().get_vfs().get_file_id(uri)
    }

    pub fn add_workspace_root(&mut self, root: PathBuf) {
        self.compilation
            .get_db_mut()
            .get_module_index_mut()
            .add_workspace_root(root);
    }

    pub fn update_file_by_uri(&mut self, uri: &Uri, text: Option<String>)  -> Option<FileId> {
        let is_removed = text.is_none();
        let file_id = self
            .compilation
            .get_db_mut()
            .get_vfs_mut()
            .set_file_content(uri, text);

        self.compilation.remove_index(vec![file_id]);
        if !is_removed {
            self.compilation.update_index(vec![file_id]);
        }

        Some(file_id)
    }

    pub fn update_file_by_path(&mut self, path: &PathBuf, text: Option<String>) -> Option<FileId> {
        let uri = file_path_to_uri(&path)?;
        self.update_file_by_uri(&uri, text)
    }

    pub fn update_files_by_uri(&mut self, files: Vec<(Uri, Option<String>)>) -> Vec<FileId> {
        let mut removed_files = Vec::new();
        let mut updated_files = Vec::new();
        for (uri, text) in files {
            let is_new_text = text.is_some();
            let file_id = self
                .compilation
                .get_db_mut()
                .get_vfs_mut()
                .set_file_content(&uri, text);
            removed_files.push(file_id);
            if is_new_text {
                updated_files.push(file_id);
            }
        }

        self.compilation.remove_index(removed_files);
        self.compilation.update_index(updated_files.clone());
        updated_files
    }

    pub fn update_files_by_path(&mut self, files: Vec<(PathBuf, Option<String>)>) -> Vec<FileId> {
        let files = files
            .into_iter()
            .filter_map(|(path, text)| {
                let uri = file_path_to_uri(&path)?;
                Some((uri, text))
            })
            .collect();
        self.update_files_by_uri(files)
    }

    pub fn update_config(&mut self, config: Arc<Emmyrc>) {
        self.compilation.update_config(config);
    }

    pub fn get_emmyrc(&self) -> Arc<Emmyrc> {
        self.emmyrc.clone()
    }
}

unsafe impl Send for EmmyLuaAnalysis {}
unsafe impl Sync for EmmyLuaAnalysis {}
