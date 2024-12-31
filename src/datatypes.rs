pub enum Type {
    Char(u16),
    Varchar(u16),
    Tiny,
    Short,
    Int,
    BigInt,
    UnsignedTiny,
    UnsignedShort,
    UnsignedInt,
    UnsignedBigInt,
    Float,
    Double,
    Decimal(u8),
    Bool,
    Enum(String),   // Lookup by Enum name
    Struct(String), // Lookup by Struct name
}

pub struct Schema {
    types: Vec<Type>,
}

pub enum Value {
    Char(String),
    Varchar(String),
    Tiny(i8),
    Short(i16),
    Int(i32),
    BigInt(i64),
    UnsignedTiny(u8),
    UnsignedShort(u16),
    UnsignedInt(u32),
    UnsignedBigInt(u64),
    Float(f32),
    Double(f64),
    Decimal(i64, u8),
    Bool(bool),
    Enum(u8, Vec<Value>), // An Enum has a numeric value, and possible data
    Struct(Vec<Value>),   // A struct is just several fields of values
}

pub struct Tuple {
    values: Vec<Value>,
}

pub struct Table {
    name: String,
    schema: Schema,
    tuples: Vec<Tuple>,
}
