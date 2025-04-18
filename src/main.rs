use std::{collections::HashMap, hash::Hash, rc::Rc};

type Id = Rc<str>;

enum Error {
    Undeclared,
}

type Result<T> = Result<T, Error>;

enum Statement {
    Let(Id, Expr),
    Assign(Id, Expr),
    Expr(Expr),

    // non-capturing unary function
    Fn(Rc<str>, Id, Expr),
}

impl Statement {
    fn infer(&self, context: &mut Context) -> Result<Type> {
        match self {
            Self::Let(id, expr) => {
                let ty = expr.infer(context)?;
                context.insert(id.clone(), ty);
            }

            Self::Assign(id, expr) => {
                let ty = expr.infer(context)?;
                context.get(&id);
            }

            Self::Expr(expr) => { expr.infer(context) }

            Self::Fn(name, param, expr) => {
                context.push();

                context.pop();
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Int(u64),
    Block(Vec<Statement>),
    Fn(Id, Expr),
    Call(Box<Self>, Box<Self>),
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
    type_var_count: usize,
    functions: Vec<HashMap<Id, TypeScheme>>,
    scopes: Vec<HashMap<Id, Type>>,
    errors: Vec<()>,
}

impl Context {
    fn new() -> Self {
        Self {
            type_var_count: 0,
            functions: vec![HashMap::new()],
            scopes: vec![HashMap::new()],
            errors: vec![],
        }
    }

    fn push() {
        self.functions.push(HashMap::new());
        self.scopes.push(HashMap::new());
    }

    fn pop() {
        self.functions.pop();
        self.scopes.pop();
    }

    fn fresh(&mut self) -> Type {
        let index = self.var_count;
        self.var_count += 1;
        Type::Var(index)
    }

    fn get(&self) -> Result<Type> {
        for function_scope in self.functions.iter().rev() {
            if let Some(schema) = function_scope.get(id) {
                return Ok(schema);
            }
        }
        Err(Error::Undeclared)
    }

    fn get_fn(&self, id: Id) -> Result<TypeScheme> {
        for function_scope in self.functions.iter().rev() {
            if let Some(schema) = function_scope.get(id) {
                return Ok(schema);
            }
        }
        Err(Error::Undeclared)
    }

    fn insert(&mut self, id: Id, ty: Type) {
        self.scopes
            .last_mut()
            .expect("should be at least one scope open")
            .insert(id.clone(), ty);
    }

    fn insert_fn(&mut self, id: Id, ty_scheme: TypeScheme) {
        self.functions
            .last_mut()
            .expect("should be at least one scope open")
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
