use crate::ast::{Atom, BinaryOp, Expr, UnaryOp};
use chumsky::Parser;
use chumsky::prelude::*;
use crate::ast::Expr::Unary;

fn boolean<'a>() -> impl Parser<char, Expr, Error=Simple<char>> {
    just("true").to(true).or(
        just("false").to(false)
    )
        .map(|b| Expr::Atom(Atom::Bool(b)))
}

#[test]
fn test_boolean_parser() {
    assert_eq!(boolean().parse("true"), Ok(Expr::Atom(Atom::Bool(true))));
    assert_eq!(boolean().parse("false"), Ok(Expr::Atom(Atom::Bool(false))));
    assert!(boolean().parse("tru").is_err());
    assert!(boolean().parse("False").is_err());
}

/// Parses floating point and integer numbers and returns them as [`Expr::Atom(Atom::Double(...))`]
/// or [`Expr::Atom(Atom::Int(...))`] types. The following formats are supported:
/// - `1`
/// - `1.`
/// - `1.0`
/// - `-1`
/// - `-1.0`
/// - `1e10`
/// - `1e-10`
/// - `1E10`
/// - `1E-10`
/// - `-1e10`
fn numbers<'a>() -> impl Parser<char, Expr, Error=Simple<char>> {
    let digits = text::digits::<char, Simple<char>>(10);

    let frac = just('.')
        .chain::<char, _, _>(digits.clone().or_not());

    let exp = just('e')
        .or(just('E'))
        .chain::<char, _, _>(
            one_of("+-").or_not()
        )
        .chain::<char, _, _>(digits.clone());


    let floating = just('-')
        .or_not()
        .chain::<char, _, _>(text::int::<char, Simple<char>>(10))
        .chain::<char, _, _>(frac.or_not().flatten())
        .chain::<char, _, _>(exp.or_not().flatten())

        .try_map(|chars, span| {
            let str = chars.into_iter().collect::<String>();

            if let Ok(i) = str.parse::<i64>() {
                Ok(Expr::Atom(Atom::Int(i)))
            } else if let Ok(f) = str.parse::<f64>() {
                Ok(Expr::Atom(Atom::Double(f)))
            } else {
                Err(Simple::expected_input_found(span, None, None))
            }
        });

    let integer = text::int(10).map(|s: String| Expr::Atom(Atom::Int(s.as_str().parse().unwrap())));

    choice((floating, integer)).padded()
}

pub fn parser<'a>() -> impl Parser<char, Expr, Error=Simple<char>> {
    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let atom = choice((
            numbers(),
            boolean()
        )
        )
            .or(expr.delimited_by(just('('), just(')')))
            .or(ident.map(Expr::Var));


        let op = |c| just::<char, _, Simple<char>>(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Expr::Unary(UnaryOp::Neg, Box::new(rhs)));

        let product_div_op = op('*').to(BinaryOp::Mul)
                .or(op('/').to(BinaryOp::Div));

        // TODO: Can't seem to clone unary here
        //let tmp = unary.clone().then(product_div_op);

        // let product = unary
        //     .then(product_div_op)
        //     .then(unary)
        //     .repeated()
        //     .foldl(|lhs, (binary_op, rhs)| Expr::Binary(Box::new(lhs),binary_op, Box::new(rhs)));
        //
        // let sum = product.clone()
        //     .then(op('+').to(Expr::Add as fn(_, _) -> _)
        //         .or(op('-').to(Expr::Sub as fn(_, _) -> _))
        //         .then(product)
        //         .repeated())
        //     .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));
        // sum
        unary
        //product
    });

    //    let decl = recursive(|decl| {
    //        let r#let = text::keyword("let")
    //            .ignore_then(ident)
    //            .then_ignore(just('='))
    //            .then(expr.clone())
    //            .then_ignore(just(';'))
    //            .then(decl)
    //            .map(|((name, rhs), then)| Expr::Let {
    //                name,
    //                rhs: Box::new(rhs),
    //                then: Box::new(then)
    //            });
    //
    //        r#let.or(expr).padded()
    //    });

    //    decl.then_ignore(end())
    expr.then_ignore(end())
}
