use crate::resolved::structure::{DataType, Scope, Variable};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Index, IndexMut};

/// Refers to a [`Scope`] stored in a [`Program`].
///
/// You'll notice that this struct requires no lifetime. This was chosen to allow for easy
/// implementation of tree-like and cyclic data resolved::structures inside the library.
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
        Program {
            scopes: vec![Scope::new()],
            entry_point: ScopeId(0),
            variables: Vec::new(),
        }
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

    pub fn get_entry_point(&self) -> ScopeId {
        self.entry_point
    }

    pub fn set_entry_point(&mut self, new_entry_point: ScopeId) {
        self.entry_point = new_entry_point;
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
}
