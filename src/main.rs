use std::{collections::HashMap, hash::Hash, rc::Rc};

type Id = Rc<str>;

enum Statement {
    Let(Id, Expr),
    Assign(Id, Expr),
    Expr(Expr),

    // non-capturing unary function
    Fn(Id, Expr),
}

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Int(u64),
    Block(Vec<Statement>),
}

impl Expr {
    fn infer(&self, _context: &mut Context) -> Result<Type, ()> {
        match self {
            Self::Int(_) => Ok(Type::Int),
            Self::Block(statements) => {
                for statement in statements {
                    match statement {
                        Statement::Let(id, expr) => {
                            let result = expr.infer(_context);
                            match result {

                            }
                        }
                    }
                }
                todo!()
            }
        }
    }
}

struct TypeScheme {
    params: usize,
    ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Var(usize),
    Int,

    // sentinel for unknown type
    Unknown,
}

struct Context {
    var_count: usize,
    functions: Vec<HashMap<Id, TypeScheme>>,
    scopes: Vec<HashMap<Id, Type>>,
    errors: Vec<()>,
}

impl Context {
    fn new() -> Self {
        Self {
            var_count: 0,
            functions: vec![HashMap::new()],
            scopes: vec![HashMap::new()],
        }
    }

    fn fresh(&mut self) -> Type {
        let index = self.var_count;
        self.var_count += 1;
        Type::Var(index)
    }

    fn insert(&mut self, id: Id, ty: Type) {
        self.scopes
            .last_mut()
            .expect("should have at least one scope open")
            .insert(id.clone(), ty);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_int() {
        let mut context = Context::new();
        assert_eq!(Expr::Int(123).infer(&mut context), Ok(Type::Int));
    }
}

fn main() {
    println!("Hello, world!");
}
