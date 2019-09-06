use crate::problem::FilePosition;
use crate::structure::{BuiltinFunction, Program, ScopeId, VarAccess, VariableId};
use crate::util::NVec;

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Automatic,
    Dynamic(VariableId),
    LoadTemplateParameter(VariableId),
    Bool,
    Int,
    Float,
    Array {
        base_type: Box<DataType>,
        sizes: Vec<VarAccess>,
    },
    Void,
    DataType_,
    Function_,
}

impl DataType {
    pub fn is_automatic(&self) -> bool {
        match self {
            DataType::Automatic => true,
            // DataType::Array(data) => data.base.is_automatic(),
            _ => false,
        }
    }

    pub fn equivalent(&self, other: &DataType, program: &Program) -> bool {
        match self {
            // If it's a basic type, just check if it is equal to the other one.
            DataType::Automatic
            | DataType::Bool
            | DataType::Int
            | DataType::Float
            | DataType::Void
            | DataType::DataType_
            | DataType::Function_ => self == other,
            // If it's a dynamic / template type, check the value contained in both targets are both
            // known and identical.
            DataType::Dynamic(target) | DataType::LoadTemplateParameter(target) => match other {
                DataType::Dynamic(other_target) | DataType::LoadTemplateParameter(other_target) => {
                    let value = program.borrow_temporary_value(*target);
                    let other_value = program.borrow_temporary_value(*other_target);
                    if let KnownData::Unknown = value {
                        false
                    } else {
                        value == other_value
                    }
                }
                _ => false,
            },
            // If it's an array type, check if the other one is also an array type. Check if both of
            // their sizes are known and are equivalent. Check if both their base types are known
            // and are equivalent.
            DataType::Array { base_type, sizes } => match other {
                DataType::Array {
                    base_type: other_base,
                    sizes: other_sizes,
                } => {
                    // Check if the base types are equivalent.
                    if !base_type.equivalent(other_base, program) {
                        return false;
                    }
                    // Check if all the sizes have known and identical values.
                    for (size, other_size) in sizes.iter().zip(other_sizes.iter()) {
                        let size_value = program.borrow_value_of(size);
                        let other_size_value = program.borrow_value_of(other_size);
                        if let KnownData::Unknown = size_value {
                            return false;
                        } else if size_value != other_size_value {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            },
        }
    }
}

impl Display for DataType {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            DataType::Automatic => write!(formatter, "Auto"),
            DataType::Dynamic(_var) => write!(formatter, "Unresolved"),
            DataType::LoadTemplateParameter(_var) => write!(formatter, "Unresolved"),
            DataType::Bool => write!(formatter, "Bool"),
            DataType::Int => write!(formatter, "Int"),
            DataType::Float => write!(formatter, "Float"),
            // TODO: Implement.
            DataType::Array { .. } => write!(formatter, "Array data type format unimplemented"),
            DataType::Void => write!(formatter, "Void"),
            DataType::DataType_ => write!(formatter, "DataType_"),
            DataType::Function_ => write!(formatter, "Function_"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionData {
    body: ScopeId,
    builtin: Option<BuiltinFunction>,
    header: FilePosition,
}

impl PartialEq for FunctionData {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.builtin == other.builtin
    }
}

impl FunctionData {
    pub fn new(body: ScopeId, header: FilePosition) -> FunctionData {
        FunctionData {
            body,
            builtin: None,
            header,
        }
    }

    pub fn builtin(body: ScopeId, builtin: BuiltinFunction, header: FilePosition) -> FunctionData {
        FunctionData {
            body,
            builtin: Option::Some(builtin),
            header,
        }
    }

    pub fn set_header(&mut self, new_header: FilePosition) {
        self.header = new_header;
    }

    pub fn get_header(&self) -> &FilePosition {
        &self.header
    }

    pub fn get_body(&self) -> ScopeId {
        self.body
    }

    pub fn is_builtin(&self) -> bool {
        self.builtin.is_some()
    }

    pub fn get_builtin(&self) -> &Option<BuiltinFunction> {
        &self.builtin
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KnownData {
    Void,
    Unknown,
    Bool(bool),
    Int(i64),
    Float(f64),
    DataType(DataType),
    Function(FunctionData),
    Array(NVec<KnownData>),
}

impl KnownData {
    pub fn new_array(dimensions: Vec<usize>) -> KnownData {
        KnownData::Array(NVec::new(dimensions, KnownData::Unknown))
    }

    pub fn require_bool(&self) -> bool {
        match self {
            KnownData::Bool(value) => *value,
            _ => panic!("Expected data to be a bool."),
        }
    }

    pub fn require_int(&self) -> i64 {
        match self {
            KnownData::Int(value) => *value,
            _ => panic!("Expected data to be an int."),
        }
    }

    pub fn require_float(&self) -> f64 {
        match self {
            KnownData::Float(value) => *value,
            _ => panic!("Expected data to be a float."),
        }
    }

    pub fn matches_data_type(&self, data_type: &DataType) -> bool {
        match self {
            KnownData::Void => {
                if let DataType::Void = data_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Unknown => false,
            KnownData::Bool(_value) => {
                if let DataType::Bool = data_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Int(_value) => {
                if let DataType::Int = data_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Float(_value) => {
                if let DataType::Float = data_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Array(contents) => {
                if let DataType::Array { base_type, .. } = data_type {
                    for item in contents.borrow_all_items() {
                        if !item.matches_data_type(base_type) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            KnownData::DataType(_value) => {
                if let DataType::DataType_ = data_type {
                    true
                } else {
                    false
                }
            }
            KnownData::Function(_value) => {
                if let DataType::Function_ = data_type {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Display for KnownData {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            KnownData::Void => write!(formatter, "[void]"),
            KnownData::Unknown => write!(formatter, "[unknown]"),
            KnownData::Bool(value) => {
                write!(formatter, "{}", if *value { "true" } else { "false" })
            }
            KnownData::Int(value) => write!(formatter, "{}", value),
            KnownData::Float(value) => write!(formatter, "{}", value),
            KnownData::Array(values) => {
                write!(formatter, "[TODO better format for N-dim arrays.\n")?;
                for value in values.borrow_all_items() {
                    write!(formatter, "\t{},\n", value)?;
                }
                write!(formatter, "]")
            }
            KnownData::DataType(value) => write!(formatter, "{}", value),
            // TODO: Implement function formatter.
            KnownData::Function(_value) => {
                write!(formatter, "[function formatter not implemented]")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    definition: FilePosition,

    data_type: DataType,
    initial_value: KnownData,
    permanent: bool,

    temporary_value: KnownData,
    // temporary_read: bool,
    // multiple_temporary_values: bool,
}

impl Variable {
    fn new_impl(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
        permanent: bool,
    ) -> Variable {
        let initial_value = initial_value.unwrap_or_else(|| KnownData::Unknown);
        Variable {
            definition,
            initial_value: initial_value.clone(),
            data_type,
            permanent,
            temporary_value: initial_value, // temporary_read: false,
                                            // multiple_temporary_values: false,
        }
    }

    pub fn variable(
        definition: FilePosition,
        data_type: DataType,
        initial_value: Option<KnownData>,
    ) -> Variable {
        Self::new_impl(definition, data_type, initial_value, false)
    }

    pub fn constant(definition: FilePosition, data_type: DataType, value: KnownData) -> Variable {
        Self::new_impl(definition, data_type, Option::Some(value), true)
    }

    pub fn function_def(function_data: FunctionData) -> Variable {
        Self::constant(
            function_data.get_header().clone(),
            DataType::Function_,
            KnownData::Function(function_data),
        )
    }

    pub fn data_type(definition: FilePosition, value: DataType) -> Variable {
        Self::constant(definition, DataType::DataType_, KnownData::DataType(value))
    }

    pub fn automatic(definition: FilePosition) -> Variable {
        Variable::variable(definition, DataType::Automatic, Option::None)
    }

    pub fn bool_literal(definition: FilePosition, value: bool) -> Variable {
        Variable::constant(definition, DataType::Bool, KnownData::Bool(value))
    }

    pub fn int_literal(definition: FilePosition, value: i64) -> Variable {
        Variable::constant(definition, DataType::Int, KnownData::Int(value))
    }

    pub fn float_literal(definition: FilePosition, value: f64) -> Variable {
        Variable::constant(definition, DataType::Float, KnownData::Float(value))
    }

    pub fn void(definition: FilePosition) -> Variable {
        Variable::variable(definition, DataType::Void, Option::None)
    }

    pub fn set_definition(&mut self, new_definition: FilePosition) {
        self.definition = new_definition;
    }

    pub fn get_definition(&self) -> &FilePosition {
        &self.definition
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
    }

    pub fn borrow_data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn set_initial_value(&mut self, value: KnownData) {
        self.initial_value = value;
    }

    pub fn borrow_initial_value(&self) -> &KnownData {
        &self.initial_value
    }

    pub fn mark_as_permanent(&mut self) {
        self.permanent = true;
    }

    pub fn is_permanent(&self) -> bool {
        self.permanent
    }

    pub fn set_temporary_value(&mut self, value: KnownData) {
        self.temporary_value = value;
    }

    pub fn borrow_temporary_value(&self) -> &KnownData {
        &self.temporary_value
    }

    pub fn borrow_temporary_value_mut(&mut self) -> &mut KnownData {
        &mut self.temporary_value
    }

    pub fn reset_temporary_value(&mut self) {
        self.temporary_value = self.initial_value.clone();
    }

    // pub fn set_temporary_value(&mut self, value: KnownData) {
    //     self.temporary_value = value;
    //     if self.temporary_read {
    //         self.multiple_temporary_values = true;
    //     }
    // }

    // pub fn borrow_temporary_value(&mut self) -> &KnownData {
    //     self.temporary_read = true;
    //     &self.temporary_value
    // }

    // pub fn reset_temporary_value(&mut self) {
    //     self.temporary_value = self.initial_value.clone();
    //     self.temporary_read = false;
    //     self.multiple_temporary_values = false;
    // }

    // pub fn had_multiple_temporary_values(&self) -> bool {
    //     self.multiple_temporary_values
    // }
}
