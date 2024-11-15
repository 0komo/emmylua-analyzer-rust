use rowan::TextRange;

use crate::InFiled;

use super::type_decl::LuaTypeDeclId;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LuaType {
    Unknown,
    Any,
    Nil,
    Table,
    Userdata,
    Thread,
    Boolean,
    String,
    Integer,
    Number,
    Io,
    SelfInfer,
    BooleanConst(bool),
    StringConst(Box<String>),
    IntegerConst(i64),
    TableConst(InFiled<TextRange>),
    Ref(LuaTypeDeclId),
    Def(LuaTypeDeclId),
    Module(Box<String>),
    Array(Box<LuaType>),
    KeyOf(Box<LuaType>),
    Nullable(Box<LuaType>),
    Tuple(Box<LuaTupleType>),
    Function(Box<LuaFunctionType>),
    Object(Box<LuaObjectType>),
    Union(Box<LuaUnionType>),
    Intersection(Box<LuaIntersectionType>),
    Extends(Box<LuaExtendedType>),
    Generic(Box<LuaGenericType>),
    TableGeneric(Box<Vec<LuaType>>),
    TplRef(usize),
    StrTplRef(Box<LuaStringTplType>),
    MuliReturn(Box<LuaMultiReturn>),
}

#[allow(unused)]
impl LuaType {
    pub fn is_unknown(&self) -> bool {
        matches!(self, LuaType::Unknown)
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, LuaType::Nil)
    }

    pub fn is_table(&self) -> bool {
        matches!(self, LuaType::Table | LuaType::TableGeneric(_) | LuaType::TableConst(_))
    }

    pub fn is_userdata(&self) -> bool {
        matches!(self, LuaType::Userdata)
    }

    pub fn is_thread(&self) -> bool {
        matches!(self, LuaType::Thread)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, LuaType::BooleanConst(_) | LuaType::Boolean)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, LuaType::StringConst(_) | LuaType::String)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, LuaType::IntegerConst(_) | LuaType::Integer)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, LuaType::Number | LuaType::Integer | LuaType::IntegerConst(_))
    }

    pub fn is_io(&self) -> bool {
        matches!(self, LuaType::Io)
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, LuaType::Ref(_))
    }

    pub fn is_def(&self) -> bool {
        matches!(self, LuaType::Def(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, LuaType::Array(_))
    }

    pub fn is_key_of(&self) -> bool {
        matches!(self, LuaType::KeyOf(_))
    }

    pub fn is_nullable(&self) -> bool {
        matches!(self, LuaType::Nullable(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, LuaType::Tuple(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, LuaType::Function(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, LuaType::Object(_))
    }

    pub fn is_union(&self) -> bool {
        matches!(self, LuaType::Union(_))
    }

    pub fn is_intersection(&self) -> bool {
        matches!(self, LuaType::Intersection(_))
    }

    pub fn is_extends(&self) -> bool {
        matches!(self, LuaType::Extends(_))
    }

    pub fn is_generic(&self) -> bool {
        matches!(self, LuaType::Generic(_) | LuaType::TableGeneric(_))
    }

    pub fn is_table_generic(&self) -> bool {
        matches!(self, LuaType::TableGeneric(_))
    }

    pub fn is_tpl_ref(&self) -> bool {
        matches!(self, LuaType::TplRef(_))
    }

    pub fn is_str_tpl_ref(&self) -> bool {
        matches!(self, LuaType::StrTplRef(_))
    }

    pub fn is_self_infer(&self) -> bool {
        matches!(self, LuaType::SelfInfer)
    }

    pub fn is_any(&self) -> bool {
        matches!(self, LuaType::Any)
    }

    pub fn is_const(&self) -> bool {
        matches!(self, LuaType::BooleanConst(_) | LuaType::StringConst(_) | LuaType::IntegerConst(_) | LuaType::TableConst(_))
    }

    pub fn is_module(&self) -> bool {
        matches!(self, LuaType::Module(_))
    }

    pub fn is_multi_return(&self) -> bool {
        matches!(self, LuaType::MuliReturn(_))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaTupleType {
    types: Vec<LuaType>,
}

impl LuaTupleType {
    pub fn new(types: Vec<LuaType>) -> Self {
        Self { types }
    }

    pub fn get_types(&self) -> &[LuaType] {
        &self.types
    }
}

impl From<LuaTupleType> for LuaType {
    fn from(t: LuaTupleType) -> Self {
        LuaType::Tuple(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaFunctionType {
    is_async: bool,
    params: Vec<(String, Option<LuaType>)>,
    ret: Vec<LuaType>,
}

impl LuaFunctionType {
    pub fn new(is_async: bool, params: Vec<(String, Option<LuaType>)>, ret: Vec<LuaType>) -> Self {
        Self {
            is_async,
            params,
            ret,
        }
    }

    pub fn is_async(&self) -> bool {
        self.is_async
    }

    pub fn get_params(&self) -> &[(String, Option<LuaType>)] {
        &self.params
    }

    pub fn get_ret(&self) -> &[LuaType] {
        &self.ret
    }
}

impl From<LuaFunctionType> for LuaType {
    fn from(t: LuaFunctionType) -> Self {
        LuaType::Function(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LuaIndexAccessKey {
    Integer(i64),
    String(String),
    Type(LuaType),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaObjectType {
    fields: Vec<(LuaIndexAccessKey, LuaType)>,
}

impl LuaObjectType {
    pub fn new(fields: Vec<(LuaIndexAccessKey, LuaType)>) -> Self {
        Self { fields }
    }

    pub fn get_fields(&self) -> &[(LuaIndexAccessKey, LuaType)] {
        &self.fields
    }
}

impl From<LuaObjectType> for LuaType {
    fn from(t: LuaObjectType) -> Self {
        LuaType::Object(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaUnionType {
    types: Vec<LuaType>,
}

impl LuaUnionType {
    pub fn new(types: Vec<LuaType>) -> Self {
        Self { types }
    }

    pub fn get_types(&self) -> &[LuaType] {
        &self.types
    }

    pub(crate) fn into_types(self) -> Vec<LuaType> {
        self.types
    }
}

impl From<LuaUnionType> for LuaType {
    fn from(t: LuaUnionType) -> Self {
        LuaType::Union(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaIntersectionType {
    types: Vec<LuaType>,
}

impl LuaIntersectionType {
    pub fn new(types: Vec<LuaType>) -> Self {
        Self { types }
    }

    pub fn get_types(&self) -> &[LuaType] {
        &self.types
    }

    pub(crate) fn into_types(self) -> Vec<LuaType> {
        self.types
    }
}

impl From<LuaIntersectionType> for LuaType {
    fn from(t: LuaIntersectionType) -> Self {
        LuaType::Intersection(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaExtendedType {
    base: LuaType,
    ext: LuaType,
}

impl LuaExtendedType {
    pub fn new(base: LuaType, ext: LuaType) -> Self {
        Self { base, ext }
    }

    pub fn get_base(&self) -> &LuaType {
        &self.base
    }

    pub fn get_ext(&self) -> &LuaType {
        &self.ext
    }
}

impl From<LuaExtendedType> for LuaType {
    fn from(t: LuaExtendedType) -> Self {
        LuaType::Extends(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaGenericType {
    base: LuaTypeDeclId,
    params: Vec<LuaType>,
}

impl LuaGenericType {
    pub fn new(base: LuaTypeDeclId, params: Vec<LuaType>) -> Self {
        Self { base, params }
    }

    pub fn get_base_type(&self) -> LuaType {
        LuaType::Ref(self.base.clone())
    }

    pub fn get_params(&self) -> &[LuaType] {
        &self.params
    }
}

impl From<LuaGenericType> for LuaType {
    fn from(t: LuaGenericType) -> Self {
        LuaType::Generic(Box::new(t))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LuaStringTplType {
    prefix: String,
    usize: usize,
}

impl LuaStringTplType {
    pub fn new(prefix: String, usize: usize) -> Self {
        Self { prefix, usize }
    }

    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    pub fn get_usize(&self) -> usize {
        self.usize
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LuaMultiReturn {
    Multi(Vec<LuaType>),
    Base(LuaType),
}

impl LuaMultiReturn {
    pub fn get_type(&self, idx: usize) -> Option<&LuaType> {
        match self {
            LuaMultiReturn::Multi(types) => types.get(idx),
            LuaMultiReturn::Base(t) => Some(t),
        }
    }
}