use std::collections::HashMap;
use crate::ast::{Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Variable {
        name: String,
        type_: Type,
        defined: bool,
        line: usize,
        column: usize,
    },
    Function {
        name: String,
        parameters: Vec<Type>,
        return_type: Type,
        line: usize,
        column: usize,
    },
    Struct {
        name: String,
        fields: HashMap<String, Type>,
        line: usize,
        column: usize,
    },
    Constant {  // Add this variant
        name: String,
        type_: Type,
        line: usize,
        column: usize,
    },
}

impl Symbol {
    pub fn get_type(&self) -> Type {
        match self {
            Symbol::Variable { type_, .. } => type_.clone(),
            Symbol::Function { return_type, .. } => return_type.clone(),
            Symbol::Struct { .. } => Type::Void, // Structs don't have a single type
            Symbol::Constant { type_, .. } => type_.clone(), // Handle Constant type
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Symbol::Constant { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
    pub children: Vec<Scope>,
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) -> bool {
        self.symbols.insert(name, symbol).is_none()
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup(name))
        })
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub current_scope: Scope,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope::new(None),
        }
    }

    pub fn enter_scope(&mut self) {
        let new_scope = Scope::new(Some(Box::new(self.current_scope.clone())));
        self.current_scope = new_scope;
    }

    pub fn leave_scope(&mut self) {
        if let Some(parent) = self.current_scope.parent.take() {
            let mut parent = *parent;
            parent.children.push(self.current_scope.clone());
            self.current_scope = parent;
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) -> bool {
        self.current_scope.insert(name, symbol)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.current_scope.lookup(name)
    }

    pub fn get_root_scope(&self) -> Scope {
        let mut current = self.current_scope.clone();
        while let Some(parent) = current.parent {
            current = *parent;
        }
        current
    }
}
