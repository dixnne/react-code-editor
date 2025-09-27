use std::collections::HashMap;
use crate::ast::{Literal, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Variable {
        name: String,
        type_: Type,
        defined: bool,
        line: usize,
        column: usize,
        value: Option<Literal>,
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
        value: Option<Literal>,
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
    pub name: String,
    pub level: usize,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
            children: Vec::new(),
            name: "".to_string(),
            level: 0,
        }
    }
}

impl Scope {
    pub fn new(parent: Option<Box<Scope>>, name: String) -> Self {
        let level = parent.as_ref().map_or(0, |p| p.level + 1);
        Scope {
            symbols: HashMap::new(),
            parent,
            children: Vec::new(),
            name,
            level,
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
            current_scope: Scope::new(None, "global".to_string()),
        }
    }

    pub fn enter_scope(&mut self, name: String) {
        let old_scope = std::mem::take(&mut self.current_scope);
        self.current_scope = Scope::new(Some(Box::new(old_scope)), name);
    }

    pub fn leave_scope(&mut self) {
        if let Some(parent) = self.current_scope.parent.take() {
            let child = std::mem::replace(&mut self.current_scope, *parent);
            self.current_scope.children.push(child);
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
