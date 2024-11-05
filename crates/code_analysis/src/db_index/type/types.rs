use super::type_decl::LuaTypeDeclId;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LuaType {
    Unknown,
    Nil,
    Table,
    Userdata,
    Thread,
    Boolean,
    String,
    Integer,
    Number,
    Io,
    BooleanConst(bool),
    StringConst(Box<String>),
    IntegerConst(i64),
    Ref(LuaTypeDeclId),
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
}

impl LuaType {
    pub fn is_unknown(&self) -> bool {
        matches!(self, LuaType::Unknown)
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, LuaType::Nil)
    }

    pub fn is_table(&self) -> bool {
        matches!(self, LuaType::Table)
    }

    pub fn is_userdata(&self) -> bool {
        matches!(self, LuaType::Userdata)
    }

    pub fn is_thread(&self) -> bool {
        matches!(self, LuaType::Thread)
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, LuaType::BooleanConst(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, LuaType::StringConst(_))
    }

    pub fn is_integer_const(&self) -> bool {
        matches!(self, LuaType::IntegerConst(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, LuaType::Integer)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, LuaType::Number)
    }

    pub fn is_io(&self) -> bool {
        matches!(self, LuaType::Io)
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, LuaType::Ref(_))
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
    params: Vec<(String, Option<LuaType>)>,
    ret: Vec<LuaType>,
}

impl LuaFunctionType {
    pub fn new(params: Vec<(String, Option<LuaType>)>, ret: Vec<LuaType>) -> Self {
        Self { params, ret }
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
