use std::collections::{HashMap, HashSet};

use super::{ast::Ast, nodes::r#type::Type};

pub struct SemanticContext {
    pub symbol_table: HashMap<String, Type>,
    pub current_scope: Vec<HashSet<String>>,
    // Optionally store additional context such as the current function's expected return type.
    pub current_function_return: Option<Type>,
}

impl SemanticContext {
    pub fn new() -> SemanticContext {
        SemanticContext {
            symbol_table: HashMap::new(),
            current_scope: vec![HashSet::new()],
            current_function_return: None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.current_scope.push(HashSet::new());
    }

    pub fn exit_scope(&mut self) {
        self.current_scope.pop();
    }

    /// Add a new symbol keyed by its unique usize id and store its Type.
    pub fn add_symbol(&mut self, id: &String, symbol_type: Type) {
        // Insert into the symbol table
        self.symbol_table.insert(id.clone(), symbol_type);
        // Record the id in the current scope for later lookup.
        if let Some(scope) = self.current_scope.last_mut() {
            scope.insert(id.clone());
        }
    }

    /// Look up a type in the symbol table by the id.
    pub fn lookup(&self, id: &String) -> Option<&Type> {
        // Check the scopes (you might simplify this if your symbol_table is global)
        for scope in self.current_scope.iter().rev() {
            if scope.contains(id) {
                return self.symbol_table.get(id);
            }
        }
        None
    }
}
pub struct SemanticAnalyzer {
    ast: Box<Ast>,
}

impl SemanticAnalyzer {
    pub fn new(ast: Box<Ast>) -> SemanticAnalyzer {
        SemanticAnalyzer { ast }
    }

    pub fn analyze(self, ctx: &mut SemanticContext) -> Result<Box<Ast>, String> {
        // Analyze each child node of the AST
        for node in self.ast.children.iter() {
            node.analyze(ctx)?;
        }

        // dbg!(&ctx.symbol_table);

        Ok(self.ast)
    }
}
