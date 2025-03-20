use std::any::Any;

#[derive(Debug)]
pub enum ValueType {
    INT,
    STRING,
    FLOAT,
    BOOL,
    TIME,
    LOCALTIME,
    LONG,
    ENUM,
}
#[derive(Debug)]
pub struct ValueItem {
    value_type: ValueType,
    value: Box<dyn Any>,
}
