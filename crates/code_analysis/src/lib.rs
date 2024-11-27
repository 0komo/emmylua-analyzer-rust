mod compilation;
mod diagnostic;
mod vfs;
mod config;
mod db_index;
mod semantic;

#[allow(unused)]
pub use compilation::*;
#[allow(unused)]
pub use diagnostic::*;
#[allow(unused)]
pub use vfs::*;
pub use config::Emmyrc;

#[derive(Debug)]
pub struct EmmyLuaAnalysis {
    compilation: LuaCompilation,
    vfs: Vfs,
}

impl EmmyLuaAnalysis {
    pub fn new() -> Self {
        Self {
            compilation: LuaCompilation::new(),
            vfs: Vfs::new(),
        }
    }
}

unsafe impl Send for EmmyLuaAnalysis {}
unsafe impl Sync for EmmyLuaAnalysis {}