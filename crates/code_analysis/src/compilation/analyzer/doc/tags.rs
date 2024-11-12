use emmylua_parser::{LuaAst, LuaAstNode, LuaAstToken, LuaClosureExpr, LuaDocTag, LuaLocalName, LuaVarExpr};

use crate::db_index::{LuaPropertyOwnerId, LuaSignatureId};

use super::{
    property_tags::{analyze_deprecated, analyze_nodiscard, analyze_source, analyze_visibility},
    type_def_tags::{analyze_alias, analyze_class, analyze_enum, analyze_func_generic},
    type_ref_tags::{analyze_module, analyze_overload, analyze_param, analyze_return, analyze_type},
    DocAnalyzer,
};

pub fn analyze_tag(analyzer: &mut DocAnalyzer, tag: LuaDocTag) -> Option<()> {
    match tag {
        // def
        LuaDocTag::Class(class) => {
            analyze_class(analyzer, class)?;
        }
        LuaDocTag::Generic(generic) => {
            analyze_func_generic(analyzer, generic)?;
        }
        LuaDocTag::Enum(enum_tag) => {
            analyze_enum(analyzer, enum_tag)?;
        }
        LuaDocTag::Alias(alias) => {
            analyze_alias(analyzer, alias)?;
        }

        // ref
        LuaDocTag::Type(type_tag) => {
            analyze_type(analyzer, type_tag)?;
        }
        LuaDocTag::Param(param_tag) => {
            analyze_param(analyzer, param_tag)?;
        }
        LuaDocTag::Return(return_tag) => {
            analyze_return(analyzer, return_tag)?;
        }
        LuaDocTag::Overload(overload_tag) => {
            analyze_overload(analyzer, overload_tag)?;
        }
        LuaDocTag::Module(module_tag) => {
            analyze_module(analyzer, module_tag)?;
        }

        // property
        LuaDocTag::Visibility(kind) => {
            analyze_visibility(analyzer, kind)?;
        }
        LuaDocTag::Source(source) => {
            analyze_source(analyzer, source)?;
        }
        LuaDocTag::Nodiscard(_) => {
            analyze_nodiscard(analyzer)?;
        }
        LuaDocTag::Deprecated(deprecated) => {
            analyze_deprecated(analyzer, deprecated)?;
        }

        _ => {}
    }

    Some(())
}

pub fn find_owner_closure(analyzer: &DocAnalyzer) -> Option<LuaClosureExpr> {
    if let Some(owner) = analyzer.comment.get_owner() {
        match owner {
            LuaAst::LuaFuncStat(func) => {
                if let Some(closure) = func.get_closure() {
                    return Some(closure);
                }
            }
            LuaAst::LuaLocalFuncStat(local_func) => {
                if let Some(closure) = local_func.get_closure() {
                    return Some(closure);
                }
            }
            owner => {
                return owner.descendants::<LuaClosureExpr>().next();
            }
        }
    }

    None
}

pub fn get_owner_id(analyzer: &mut DocAnalyzer) -> Option<LuaPropertyOwnerId> {
    let owner = analyzer.comment.get_owner()?;
    match owner {
        LuaAst::LuaLocalFuncStat(_) | LuaAst::LuaFuncStat(_) => {
            let closure = find_owner_closure(analyzer)?;
            Some(LuaPropertyOwnerId::Signature(LuaSignatureId::new(
                analyzer.file_id,
                &closure,
            )))
        },
        LuaAst::LuaAssignStat(assign) => {
            let first_var = assign.child::<LuaVarExpr>()?;
            match first_var {
                LuaVarExpr::NameExpr(name_expr) => {
                    let name = name_expr.get_name_text()?;
                    let decl = analyzer
                        .db
                        .get_decl_index()
                        .get_decl_tree(&analyzer.file_id)?
                        .find_local_decl(&name, name_expr.get_position())?;
                    return Some(LuaPropertyOwnerId::LuaDecl(decl.get_id()));
                }
                _ => None,
            }
        },
        LuaAst::LuaLocalStat(local_stat) => {
            let local_name = local_stat.child::<LuaLocalName>()?;
            let name_token = local_name.get_name_token()?;
            let name = name_token.get_name_text();
            let decl = analyzer
                .db
                .get_decl_index()
                .get_decl_tree(&analyzer.file_id)?
                .find_local_decl(&name, name_token.get_position())?;
            return Some(LuaPropertyOwnerId::LuaDecl(decl.get_id()));
        }
        _ => None,
    }
}