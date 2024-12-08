use crate::DiagnosticCode;

use super::{DiagnosticContext, LuaChecker};

#[derive(Debug)]
pub struct TypeNotFoundChecker();

impl LuaChecker for TypeNotFoundChecker {
    fn check(&self, context: &mut DiagnosticContext) -> Option<()> {
        let errors = {
            let db = context.get_db();
            let file_id = context.get_file_id();
            let diagnostic_index = db.get_diagnostic_index();
            let errors = diagnostic_index.get_diagnostics(file_id)?;
            let mut analyze_errs = Vec::new();
            for error in errors {
                if error.kind == DiagnosticCode::TypeNotFound {
                    analyze_errs.push((
                        error.message.clone(),
                        error.range.clone(),
                    ));
                }
            }

            analyze_errs
        };
        
        for analyze_err in errors {
            context.add_diagnostic(
                DiagnosticCode::TypeNotFound,
                analyze_err.1,
                analyze_err.0,
                None,
            );
        }
        Some(())
    }

    fn get_code(&self) -> DiagnosticCode {
        DiagnosticCode::TypeNotFound
    }
}
