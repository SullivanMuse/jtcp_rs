use std::collections::HashMap;

struct Flags {
    polymorphic_let: bool,
}

impl Flags {
    fn all() -> Self {
        Self {
            polymorphic_let: true,
        }
    }

    fn none() -> Self {
        Self {
            polymorphic_let: false,
        }
    }
}

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
    Id(Id<'id>),                                      // x
    Fn(Id<'id>, Box<Self>),                           // x -> x
    Let(Id<'id>, Vec<Id<'id>>, Box<Self>, Box<Self>), // let f x y = v; b
    Call(Box<Self>, Box<Self>),                       // f x
}

type Var = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Var(Var),
    Fn(Box<Self>, Box<Self>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Scheme {
    bounds: Vec<Var>,
    ty: Type,
}

type Unifier = HashMap<Var, Type>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Context<'id> {
    vars: usize,
    scopes: Vec<HashMap<Id<'id>, Scheme>>,
}

// functions
impl<'id> Expr<'id> {
    fn infer(&self, context: &mut Context<'id>, flags: &Flags) -> Result<Type, Error> {
        match self {
            Self::Id(id) => context.get(id).map(|ty| ty.clone()),
            Self::Let(key, params, value, body) => {
                context.enter();
                let scheme = {
                    let prev_vars = context.vars;
                    context.enter();
                    let mut bounds = Vec::new();
                    for p in params {
                        let var = context.fresh();
                        bounds.push(var.clone());
                        let scheme = Scheme::from(Type::Var(var));
                        context.insert(p, scheme);
                    }
                    let ty = value.infer(context, flags)?;
                    context.exit();
                    let out = Scheme { bounds, ty };
                    context.vars = prev_vars;
                    out
                };
                context.insert(key, scheme);
                let result = body.infer(context, flags);
                context.exit();
                result
            }
            Self::Fn(x, body) => {
                context.enter();
                let k = context.fresh();
                context.insert(x, Scheme::from(Type::Var(k)));
                let ty = body.infer(context, flags)?;
                context.exit();
                Ok(Type::Fn(Box::new(Type::Var(k)), Box::new(ty)))
            }
            Self::Call(f, x) => {
                let f_ty = f.infer(context, flags)?;
                if let Type::Fn(param_ty, mut body_ty) = f_ty {
                    let x_ty = x.infer(context, flags)?;
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

impl Scheme {
    fn from(ty: Type) -> Self {
        Self {
            bounds: Vec::new(),
            ty,
        }
    }
}

impl<'id> Context<'id> {
    fn new() -> Self {
        Self {
            vars: 0,
            scopes: Vec::new(),
        }
    }

    fn last(&self) -> &HashMap<Id<'id>, Scheme> {
        self.scopes.last().expect("never entered a scope")
    }

    fn last_mut(&mut self) -> &mut HashMap<Id<'id>, Scheme> {
        self.scopes.last_mut().expect("never entered a scope")
    }

    fn enter(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit(&mut self) {
        self.scopes.pop().expect("never entered a scope");
    }

    fn fresh(&mut self) -> Var {
        let out = self.vars;
        self.vars += 1;
        out
    }

    fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let mut unifier = HashMap::new();
        for b in &scheme.bounds {
            let f = self.fresh();
            unifier.insert(*b, Type::Var(f));
        }
        let mut ty = scheme.ty.clone();
        ty.subst(&unifier);
        ty
    }

    fn get(&mut self, id: Id<'id>) -> Result<Type, Error> {
        let mut result: Option<Scheme> = None;
        for scope in self.scopes.iter().rev() {
            if let Some(scheme) = scope.get(id) {
                result = Some(scheme.clone());
                break;
            }
        }
        if let Some(scheme) = result {
            let ty = self.instantiate(&scheme);
            return Ok(ty);
        }

        Err(Error::Undefined)
    }

    fn insert(&mut self, id: Id<'id>, scheme: Scheme) {
        self.last_mut().insert(id, scheme);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undefined1() {
        let mut context = Context::new();
        let e = Expr::Id("xyz");
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
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
                Vec::new(),
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("x")),
                )),
            )),
        );
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
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
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
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
        let e = Expr::Let("id", Vec::new(), Box::new(id), Box::new(Expr::Id("id")));
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
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
                Vec::new(),
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("id")),
                )),
            )),
        );
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
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
                Vec::new(),
                Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
                Box::new(Expr::Call(
                    Box::new(Expr::Id("id")),
                    Box::new(Expr::Id("y")),
                )),
            )),
        );
        let flags = Flags::all();
        let result = e.infer(&mut context, &flags);
        assert_eq!(
            result,
            Ok(Type::Fn(Box::new(Type::Var(0)), Box::new(Type::Var(0)))),
            "ident of var has var type"
        )
    }

    #[test]
    fn test_polymorphic_id() {
        // let id = x -> x,
        //     first = x y -> x;
        // first (id id) (id first)
        //     => a -> a
        let mut context = Context::new();
        let expr = Expr::Let(
            "id",
            vec![],
            Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
            Box::new(Expr::Let(
                "first",
                vec![],
                Box::new(Expr::Fn(
                    "x",
                    Box::new(Expr::Fn("y", Box::new(Expr::Id("x")))),
                )),
                Box::new(Expr::Call(
                    Box::new(Expr::Call(
                        Box::new(Expr::Id("first")),
                        Box::new(Expr::Call(
                            Box::new(Expr::Id("id")),
                            Box::new(Expr::Id("id")),
                        )),
                    )),
                    Box::new(Expr::Call(
                        Box::new(Expr::Id("id")),
                        Box::new(Expr::Id("first")),
                    )),
                )),
            )),
        );
        let flags = Flags::all();
        let result = expr.infer(&mut context, &flags);
        assert_eq!(
            result,
            Ok(Type::Fn(Box::new(Type::Var(0)), Box::new(Type::Var(0)))),
            "type checker supports polymorphic identifier function"
        )
    }
}

fn main() {
    // y -> let id = x -> x; id x
    let mut context = Context::new();
    let expr = Expr::Fn(
        "y",
        Box::new(Expr::Let(
            "id",
            Vec::new(),
            Box::new(Expr::Fn("x", Box::new(Expr::Id("x")))),
            Box::new(Expr::Call(
                Box::new(Expr::Id("id")),
                Box::new(Expr::Id("x")),
            )),
        )),
    );
    let flags = Flags::all();
    let result = expr.infer(&mut context, &flags);
    let _ = dbg!(result);
}
