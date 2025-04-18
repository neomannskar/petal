use std::{collections::HashMap, rc::Rc};

use crate::{
    front::nodes::node::Node,
    middle::ir::{IRContext, IRInstruction},
};

use super::semantic::SemanticContext;

pub struct Ast {
    pub children: Vec<Box<dyn Node>>,
    pub ids: HashMap<String, Rc<Box<dyn Node>>>,
}

impl Node for Ast {
    fn display(&self, indentation: usize) {
        println!("{:>width$}Abstract Syntax Tree", "", width = indentation);
        for child in &self.children {
            child.display(indentation);
        }
    }

    fn analyze(&self, ctx: &mut SemanticContext) -> Result<(), String> {
        Ok(())
    }

    fn ir(&self, ctx: &mut IRContext) -> Vec<IRInstruction> {
        let mut instructions = Vec::new();

        // Generate IR for parameters
        for child in &self.children {
            instructions.extend(child.ir(ctx));
        }

        instructions
    }
}

impl Node for Box<Ast> {
    fn display(&self, indentation: usize) {
        println!(
            "{:>width$}Abstract Syntax Tree\n┌───────────────────",
            "",
            width = indentation
        );
        for child in &self.children {
            child.display(indentation);
        }
    }

    fn analyze(&self, ctx: &mut SemanticContext) -> Result<(), String> {
        Ok(())
    }

    fn ir(&self, ctx: &mut IRContext) -> Vec<IRInstruction> {
        let mut instructions = Vec::new();

        // Generate IR for parameters
        for child in &self.children {
            instructions.extend(child.ir(ctx));
        }

        instructions
    }
}

impl Ast {
    pub fn new() -> Ast {
        Ast {
            children: Vec::new(),
            ids: HashMap::new(),
        }
    }
}
