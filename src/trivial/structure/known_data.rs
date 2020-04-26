use super::{DataType};
use crate::util::{self, NVec};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq)]
pub enum KnownData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<KnownData>),
}

impl Debug for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "{}i32", value),
            Self::Float(value) => write!(formatter, "{}f32", value),
            Self::Bool(value) => write!(formatter, "{}b1", if *value { "true" } else { "false" }),
            Self::Array(values) => {
                write!(formatter, "[")?;
                if values.len() > 0 {
                    for value in &values[..values.len() - 1] {
                        write!(formatter, "{:?}, ", value)?;
                    }
                    write!(formatter, "{:?}", values[values.len() - 1])?;
                }
                write!(formatter, "]")
            }
        }
    }
}

impl KnownData {
    // pub fn from_binary_data(target_type: &DataType, data: &[u8]) -> KnownData {
    //     // assert!(target_type.get_physical_size() == data.len());
    //     if target_type.borrow_dimensions().len() > 0 {
    //         let dims: Vec<_> = target_type.borrow_dimensions().clone();
    //         let num_items = dims.iter().fold(1, |num, dim| num * dim);
    //         let item_type = target_type.clone_and_unwrap(dims.len());
    //         let item_size = item_type.get_physical_size();
    //         let mut items = Vec::with_capacity(num_items);
    //         for index in 0..num_items {
    //             items.push(KnownData::from_binary_data(
    //                 &item_type,
    //                 &data[index * item_size..(index + 1) * item_size],
    //             ));
    //         }
    //         KnownData::Array(NVec::from_vec_and_dims(items, dims))
    //     } else {
    //         match target_type.get_base() {
    //             BaseType::B8 => Self::Bool(data[0] > 0),
    //             BaseType::I32 => {
    //                 Self::Int(i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64)
    //             }
    //             BaseType::F32 => {
    //                 Self::Float(f32::from_le_bytes([data[0], data[1], data[2], data[3]]) as f64)
    //             }
    //         }
    //     }
    // }
    pub fn build_array(dimensions: &[usize], element: KnownData) -> KnownData {
        if dimensions.len() == 0 {
            element
        } else {
            let flat_element = Self::build_array(&dimensions[1..], element);
            KnownData::Array(vec![flat_element; dimensions[0]])
        }
    }

    pub fn get_type(&self) -> DataType {
        match self {
            Self::Int(..) => DataType::i32_scalar(),
            Self::Float(..) => DataType::f32_scalar(),
            Self::Bool(..) => DataType::b1_scalar(),
            Self::Array(data) => {
                let dtype = data[0].get_type();
                dtype.wrap_with_dimension(data.len())
            }
        }
    }

    pub fn binary_data(&self) -> u32 {
        match self {
            Self::Bool(value) => {
                if *value {
                    1
                } else {
                    0
                }
            }
            Self::Int(value) => *value as i32 as u32,
            Self::Float(value) => f32::to_bits(*value as f32),
            Self::Array(..) => unimplemented!(),
        }
    }

    fn add_binary_data(&self, to: &mut Vec<u8>) {
        match self {
            Self::Bool(value) => to.push(if *value { 1 } else { 0 }),
            Self::Int(value) => {
                for byte in (*value as i32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Float(value) => {
                for byte in (*value as f32).to_le_bytes().iter() {
                    to.push(*byte);
                }
            }
            Self::Array(values) => {
                for value in values {
                    value.add_binary_data(to);
                }
            }
        }
    }

    pub fn arbitrary_len_binary_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        self.add_binary_data(&mut data);
        data
    }

    pub fn require_int(&self) -> i64 {
        if let Self::Int(value) = self {
            *value
        } else {
            panic!("Required an Int, but got a {:?}.", self)
        }
    }

    pub fn require_float(&self) -> f64 {
        if let Self::Float(value) = self {
            *value
        } else {
            panic!("Required an Float, but got a {:?}.", self)
        }
    }
}
