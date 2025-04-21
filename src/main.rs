use std::collections::HashMap;

// types
#[derive(Clone, Debug, PartialEq, Eq)]
enum Error {
    Undefined,
    ExpectedFn,
    Unification,
}

type Id<'id> = &'id str;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr<'id> {
    Id(Id<'id>),
    Fn(Id<'id>, Box<Self>),
    Let(Id<'id>, Box<Self>, Box<Self>),
    Call(Box<Self>, Box<Self>),
}

type Var = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Var(Var),
    Fn(Box<Self>, Box<Self>),
}

type Unifier = HashMap<Var, Type>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Context<'id> {
    vars: usize,
    types: Vec<(Id<'id>, Type)>,
}

// functions
impl<'id> Expr<'id> {
    fn infer(&self, context: &mut Context<'id>) -> Result<Type, Error> {
        match self {
            Self::Id(id) => context.get(id).map(|ty| ty.clone()),
            Self::Let(k, v, body) => {
                let ty = v.infer(context)?;
                context.insert(k, ty);
                let result = body.infer(context);
                context.pop();
                result
            }
            Self::Fn(x, body) => {
                let k = context.fresh();
                context.insert(x, k.clone());
                let ty = body.infer(context)?;
                context.pop();
                Ok(Type::Fn(Box::new(k), Box::new(ty)))
            }
            Self::Call(f, x) => {
                let f_ty = f.infer(context)?;
                if let Type::Fn(param_ty, mut body_ty) = f_ty {
                    let x_ty = x.infer(context)?;
                    let mut unifier = HashMap::new();
                    param_ty.unify(&x_ty, &mut unifier)?;
                    body_ty.subst(&unifier);
                    Ok(*body_ty)
                } else {
                    Err(Error::ExpectedFn)
                }
            }
        }
    }
}

impl Type {
    fn unify(&self, other: &Self, unifier: &mut Unifier) -> Result<(), Error> {
        match (self, other) {
            (Self::Var(v1), other) => {
                if let Self::Var(v2) = other {
                    if v1 == v2 {
                        return Ok(());
                    }
                }
                unifier.insert(*v1, other.clone());
                Ok(())
            }
            (Self::Fn(k1, v1), Self::Fn(k2, v2)) => {
                k1.unify(k2, unifier)?;
                v1.unify(v2, unifier)?;
                Ok(())
            }
            _ => Err(Error::Unification),
        }
    }

    fn subst(&mut self, unifier: &Unifier) {
        match self {
            Type::Var(var) => {
                if let Some(ty) = unifier.get(var) {
                    *self = ty.clone();
                }
            }
            Type::Fn(k, v) => {
                k.subst(unifier);
                v.subst(unifier);
            }
        }
    }
}

impl<'id> Context<'id> {
    fn new() -> Self {
        Self {
            vars: 0,
            types: Vec::new(),
        }
    }

    fn fresh(&mut self) -> Type {
        let out = self.vars;
        self.vars += 1;
        Type::Var(out)
    }

    fn get(&self, id: Id<'id>) -> Result<&Type, Error> {
        for (k, v) in self.types.iter().rev() {
            if *k == id {
                return Ok(v);
            }
        }
        Err(Error::Undefined)
    }

    fn pop(&mut self) {
        self.types.pop().expect("pop must follow insert");
    }

    fn insert(&mut self, id: Id<'id>, ty: Type) {
        self.types.push((id, ty));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undefined1() {
        let mut context = Context::new();
        let e = Expr::Id("xyz");
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Err(Error::Undefined),
            "use of undefined variable results in type error"
        );
    }

    #[test]
    fn test_undefined2() {
        // y -> let id = x -> x; id x
        let mut context = Context::new();
        let e = Expr::Fn(
            "y",
            Box::new(Expr::Let(
                "id",
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("x")),
                )),
            )),
        );
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Err(Error::Undefined),
            "use of variable defined in inner scope"
        );
    }

    #[test]
    fn infer_ident_fn() {
        let mut context = Context::new();
        let e = Expr::Fn("x", Box::new(Expr::Id("x")));
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Ok(Type::Fn(Box::new(Type::Var(0)), Box::new(Type::Var(0)))),
            "identity function has correct type"
        );
    }

    #[test]
    fn test_let() {
        let mut context = Context::new();
        let id = Expr::Fn("x", Box::new(Expr::Id("x")));
        let e = Expr::Let("id", Box::new(id), Box::new(Expr::Id("id")));
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Ok(Type::Fn(Box::new(Type::Var(0)), Box::new(Type::Var(0)))),
            "use of let var results in type substitution"
        );
    }

    #[test]
    fn test_identity_identity() {
        // y -> let id = x -> x; id id
        let mut context = Context::new();
        let e = Expr::Fn(
            "y",
            Box::new(Expr::Let(
                "id",
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("id")),
                )),
            )),
        );
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Ok(Type::Fn(
                Box::new(Type::Var(0)),
                Box::new(Type::Fn(Box::new(Type::Var(1)), Box::new(Type::Var(1))))
            )),
            "ident of ident has ident type"
        )
    }

    #[test]
    fn test_identity_application() {
        // y -> let id = x -> x; id y
        let mut context = Context::new();
        let e = Expr::Fn(
            "y",
            Box::new(Expr::Let(
                "id",
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("y")),
                )),
            )),
        );
        let result = e.infer(&mut context);
        assert_eq!(
            result,
            Ok(Type::Fn(Box::new(Type::Var(0)), Box::new(Type::Var(0)))),
            "ident of var has var type"
        )
    }
}

fn main() {
    // y -> let id = x -> x; id x
    let mut context = Context::new();
    let e = Expr::Fn(
        "y",
        Box::new(Expr::Let(
            "id",
            Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
            Box::new(Expr::Call(
                Box::new(Expr::Id("id")),
                Box::new(Expr::Id("x")),
            )),
        )),
    );
    let result = e.infer(&mut context);
    let _ = dbg!(result);
}
