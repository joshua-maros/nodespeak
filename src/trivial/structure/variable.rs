use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum DataType {
    B1,
    I32,
    F32,
    Array(usize, Box<DataType>),
}

impl Debug for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::B1 => write!(formatter, "b1"),
            Self::I32 => write!(formatter, "i32"),
            Self::F32 => write!(formatter, "f32"),
            Self::Array(len, base) => write!(formatter, "[{}]{:?}", len, base),
        }
    }
}

#[derive(Clone)]
pub struct Variable {
    typ: DataType,
}

impl Debug for Variable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.typ)
    }
}

impl Variable {
    pub fn new(typ: DataType) -> Variable {
        Variable { typ }
    }

    pub fn borrow_type(&self) -> &DataType {
        &self.typ
    }
}
