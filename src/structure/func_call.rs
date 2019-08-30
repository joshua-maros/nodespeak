use crate::problem::FilePosition;
use crate::structure::{DataType, Program, VariableId};

#[derive(Clone, Debug, PartialEq)]
pub struct VarAccess {
    base: VariableId,
    indexes: Vec<VariableId>,
}

impl VarAccess {
    pub fn new(base: VariableId) -> VarAccess {
        VarAccess {
            base: base,
            indexes: Vec::new(),
        }
    }

    pub fn add_index(&mut self, index: VariableId) {
        self.indexes.push(index);
    }

    pub fn borrow_indexes(&self) -> &Vec<VariableId> {
        &self.indexes
    }

    pub fn get_base(&self) -> VariableId {
        self.base
    }

    pub fn iterate_over_indexes(&self) -> std::slice::Iter<VariableId> {
        self.indexes.iter()
    }

    pub fn borrow_data_type<'a>(&'a self, program: &'a Program) -> &'a DataType {
        let data_type = program.borrow_variable(self.base).borrow_data_type();
        // For now, we don't have any code to manage arrays because arrays didn't exist at the time
        // of writing this code.
        assert!(self.indexes.len() == 0);
        data_type
    }
}

#[derive(Clone, Debug)]
pub struct FuncCall {
    function: VariableId,
    position: FilePosition,
    inputs: Vec<VarAccess>,
    outputs: Vec<VarAccess>,
}

impl FuncCall {
    pub fn new(function: VariableId, position: FilePosition) -> FuncCall {
        FuncCall {
            function: function,
            position,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn set_position(&mut self, new_position: FilePosition) {
        self.position = new_position;
    }

    pub fn get_position(&self) -> &FilePosition {
        &self.position
    }

    pub fn set_function(&mut self, function: VariableId) {
        self.function = function;
    }

    pub fn add_input(&mut self, input: VarAccess) {
        self.inputs.push(input);
    }

    pub fn borrow_inputs(&self) -> &Vec<VarAccess> {
        &self.inputs
    }

    pub fn add_output(&mut self, output: VarAccess) {
        self.outputs.push(output);
    }

    pub fn borrow_outputs(&self) -> &Vec<VarAccess> {
        &self.outputs
    }

    pub fn get_function(&self) -> VariableId {
        self.function
    }

    pub fn iterate_over_inputs(&self) -> std::slice::Iter<VarAccess> {
        self.inputs.iter()
    }

    pub fn iterate_over_outputs(&self) -> std::slice::Iter<VarAccess> {
        self.outputs.iter()
    }
}
