use crate::problem::FilePosition;
use crate::vague::structure::{
    self, Builtins, DataType, Expression, FunctionData, KnownData, Scope, Variable,
};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data vague::structures inside the library.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ScopeId(usize);

impl Debug for ScopeId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "s{}", self.0)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VariableId(usize);

impl Debug for VariableId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "v{}", self.0)
    }
}

/// Represents an entire program written in the Waveguide language.
pub struct Program {
    scopes: Vec<Scope>,
    entry_point: ScopeId,
    variables: Vec<Variable>,
    builtins: Option<Builtins>,
}

impl Debug for Program {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "entry point: {:?}", self.entry_point)?;
        for (index, scope) in self.scopes.iter().enumerate() {
            write!(formatter, "\ncontents of {:?}:\n", ScopeId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", scope).replace("\n", "\n    ")
            )?;
        }
        for (index, variable) in self.variables.iter().enumerate() {
            write!(formatter, "\ndetails for {:?}:\n", VariableId(index))?;
            write!(
                formatter,
                "    {}",
                format!("{:?}", variable).replace("\n", "\n    ")
            )?;
        }
        write!(formatter, "")
    }
}

impl Index<ScopeId> for Program {
    type Output = Scope;

    fn index(&self, scope: ScopeId) -> &Self::Output {
        &self.scopes[scope.0]
    }
}

impl IndexMut<ScopeId> for Program {
    fn index_mut(&mut self, scope: ScopeId) -> &mut Self::Output {
        &mut self.scopes[scope.0]
    }
}

impl Index<VariableId> for Program {
    type Output = Variable;

    fn index(&self, variable: VariableId) -> &Self::Output {
        &self.variables[variable.0]
    }
}

impl IndexMut<VariableId> for Program {
    fn index_mut(&mut self, variable: VariableId) -> &mut Self::Output {
        &mut self.variables[variable.0]
    }
}

impl Program {
    pub fn new() -> Program {
        let mut prog = Program {
            scopes: vec![Scope::new()],
            entry_point: ScopeId(0),
            variables: Vec::new(),
            builtins: Option::None,
        };
        prog.builtins = Option::Some(structure::add_builtins(&mut prog));
        prog
    }

    /// Creates a new scope that has no parent.
    pub fn create_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new());
        id
    }

    pub fn create_child_scope(&mut self, parent: ScopeId) -> ScopeId {
        assert!(parent.0 < self.scopes.len());
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::from_parent(parent));
        id
    }

    // ===SYMBOLS/VARIABLES=========================================================================
    pub fn lookup_symbol(&self, scope: ScopeId, symbol: &str) -> Option<VariableId> {
        match self[scope].borrow_symbols().get(symbol) {
            Option::Some(value_) => Option::Some(*value_),
            Option::None => match self[scope].get_parent() {
                Option::Some(parent) => self.lookup_symbol(parent, symbol),
                Option::None => Option::None,
            },
        }
    }

    pub fn modify_variable(&mut self, variable: VariableId, modified: Variable) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0] = modified;
    }

    pub fn adopt_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(self.variables.len());
        self.variables.push(variable);
        id
    }

    pub fn set_data_type(&mut self, variable: VariableId, data_type: DataType) {
        assert!(variable.0 < self.variables.len());
        self.variables[variable.0].set_data_type(data_type);
    }

    pub fn borrow_value_of<'a>(&'a self, expression: &'a Expression) -> &'a KnownData {
        debug_assert!(expression.is_valid());
        match expression {
            Expression::Access { base, indexes, .. } => {
                // TODO handle expressions that need to be interpreted before we can determine
                // their value.
                let mut real_indexes = Vec::new();
                for index in indexes {
                    match self.borrow_value_of(index) {
                        // TODO check that value is positive.
                        KnownData::Int(value) => real_indexes.push(*value as usize),
                        _ => panic!("TODO error, indexes must be known integers."),
                    }
                }
                let data = self.borrow_value_of(base.borrow());
                if let KnownData::Array(contents) = data {
                    if !contents.is_inside(&real_indexes) {
                        panic!("TODO error, indexes must be inside array bounds.")
                    }
                    contents.borrow_item(&real_indexes)
                } else {
                    panic!("TODO error, base of access expression is not array.")
                }
            }
            Expression::Literal(data, ..) => data,
            Expression::Variable(id, ..) => self[*id].borrow_temporary_value(),
            _ => panic!("TODO: error, cannot borrow value of complex expression."),
        }
    }

    pub fn borrow_value_of_mut(&mut self, expression: &Expression) -> &mut KnownData {
        debug_assert!(expression.is_valid());
        match expression {
            Expression::Access { base, indexes, .. } => {
                // TODO handle expressions that need to be interpreted before we can determine
                // their value.
                let mut real_indexes = Vec::new();
                for index in indexes {
                    match self.borrow_value_of(index) {
                        // TODO check that value is positive.
                        KnownData::Int(value) => real_indexes.push(*value as usize),
                        _ => panic!("TODO error, indexes must be known integers."),
                    }
                }
                let data = self.borrow_value_of_mut(base.borrow());
                if let KnownData::Array(contents) = data {
                    if !contents.is_inside(&real_indexes) {
                        panic!("TODO error, indexes must be inside array bounds.")
                    }
                    contents.borrow_item_mut(&real_indexes)
                } else {
                    panic!("TODO error, base of access expression is not array.")
                }
            }
            Expression::Literal(..) => panic!("TODO error, cannot borrow literal as mutable."),
            Expression::Variable(id, ..) => self[*id].borrow_temporary_value_mut(),
            _ => panic!("TODO: error, cannot borrow value of complex expression."),
        }
    }

    pub fn set_value_of(&mut self, expression: &Expression, value: KnownData) {
        (*self.borrow_value_of_mut(expression)) = value;
    }

    /// Resets the temporary value of every variable to their permanent value values.
    ///
    /// This should be used before beginning interpretation or simplification to reset all values
    /// to a value they are known to have at the beginning of the program.
    pub fn reset_all_temporary_value(&mut self) {
        for variable in self.variables.iter_mut() {
            variable.reset_temporary_value();
        }
    }

    pub fn get_entry_point(&self) -> ScopeId {
        self.entry_point
    }

    pub fn set_entry_point(&mut self, new_entry_point: ScopeId) {
        self.entry_point = new_entry_point;
    }

    pub fn get_builtins(&self) -> &Builtins {
        self.builtins.as_ref().unwrap()
    }

    pub fn adopt_and_define_symbol(
        &mut self,
        scope: ScopeId,
        symbol: &str,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self[scope].define_symbol(symbol, id);
        id
    }

    pub fn adopt_and_define_intermediate(
        &mut self,
        scope: ScopeId,
        definition: Variable,
    ) -> VariableId {
        let id = self.adopt_variable(definition);
        self[scope].define_intermediate(id);
        id
    }

    pub fn make_intermediate_auto_var(
        &mut self,
        scope: ScopeId,
        position: FilePosition,
    ) -> VariableId {
        self.adopt_and_define_intermediate(scope, Variable::automatic(position))
    }

    // Tries to find a function variable which uses the specified scope as a
    // function body. If nothing is found, Option::None is returned. Note that
    // while the data in the variable is guaranteed to contain correct
    // information about e.g. the inputs and outputs of the function, it is not
    // guaranteed to be the actual original variable that described the function
    // when the program was first parsed.
    pub fn find_parent_function(&self, scope: ScopeId) -> Option<VariableId> {
        // TODO: This is very inefficient.
        // TODO: This function might be useless and buggy, need to review how it
        // is used in the rest of the code.
        let mut real_scope = scope.0;
        loop {
            let mut index: usize = 0;
            for variable in self.variables.iter() {
                if variable.is_permanent() {
                    if let KnownData::Function(data) = variable.borrow_initial_value() {
                        if data.get_body() == scope {
                            return Option::Some(VariableId(index));
                        }
                    }
                }
                index += 1;
            }
            match self.scopes[real_scope].get_parent() {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.0,
            };
        }
    }

    // Tries to find a function variable which uses the specified scope as a
    // function body. If nothing is found, Option::None is returned.
    pub fn lookup_and_clone_parent_function(&self, scope: ScopeId) -> Option<FunctionData> {
        // TODO: This is very inefficient.
        // TODO: This function might be useless and buggy, need to review how it
        // is used in the rest of the code.
        let mut real_scope = scope.0;
        loop {
            for variable in self.variables.iter() {
                if variable.is_permanent() {
                    if let KnownData::Function(data) = variable.borrow_initial_value() {
                        if data.get_body() == scope {
                            return Option::Some(data.clone());
                        }
                    }
                }
            }
            match self.scopes[real_scope].get_parent() {
                Option::None => return Option::None,
                Option::Some(id) => real_scope = id.0,
            };
        }
    }
}