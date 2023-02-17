mod macros;
mod url;

use chumsky::{
    prelude::Simple,
    primitive::{choice, end, filter, just},
    recursive::recursive,
    Error, Parser,
};

use crate::macros::set;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub extras: Vec<String>,
    pub spec: Option<Spec>,
    pub marker: Option<Marker>,
}

#[derive(Clone, Debug)]
pub enum Spec {
    Url(String),
    Version(Vec<VersionSpec>),
}

#[derive(Clone, Debug)]
pub struct VersionSpec {
    pub comparator: Comparator,
    pub version: String,
}

#[derive(Clone, Debug)]
pub enum Marker {
    And(Box<Marker>, Box<Marker>),
    Or(Box<Marker>, Box<Marker>),
    Operator(Variable, Operator, Variable),
}

#[derive(Clone, Debug)]
pub enum Variable {
    PythonVersion,
    PythonFullVersion,
    OsName,
    SysPlatform,
    PlatformRelease,
    PlatformSystem,
    PlatformVersion,
    PlatformMachine,
    PlatformPythonImplementation,
    ImplementationName,
    ImplementationVersion,
    Extra,
    String(String),
}

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Comparator(Comparator),
    In,
    NotIn,
}

#[derive(Clone, Copy, Debug)]
pub enum Comparator {
    Lt, // <
    Le, // <=
    Ne, // !=
    Eq, // ==
    Ge, // >=
    Gt, // >
    Cp, // ~=
    Ae, // ===
}

pub fn parse(s: &str) -> Result<Dependency, Vec<Simple<char>>> {
    parser().then_ignore(end()).parse(s)
}

pub fn parser<E: Error<char> + 'static>() -> impl Parser<char, Dependency, Error = E> {
    let ws = set!(' ' | '\t').repeated().ignored();
    let ident = filter(char::is_ascii_alphanumeric)
        .chain(
            set!('-' | '_' | '.')
                .or_not()
                .chain(filter(char::is_ascii_alphanumeric))
                .repeated()
                .flatten(),
        )
        .collect();

    let cmp = choice((
        just("===").to(Comparator::Ae),
        just("<=").to(Comparator::Le),
        just("!=").to(Comparator::Ne),
        just("==").to(Comparator::Eq),
        just(">=").to(Comparator::Ge),
        just("~=").to(Comparator::Cp),
        just('<').to(Comparator::Lt),
        just('>').to(Comparator::Gt),
    ));

    let version_spec = cmp
        .then_ignore(ws)
        .then(
            set!(
                'A' ..= 'Z' | 'a' ..= 'z' | '0' ..= '9' | '-' | '_' | '.' | '*' | '+' | '!'
            )
            .repeated()
            .at_least(1)
            .collect(),
        )
        .map(|(comparator, version)| VersionSpec {
            comparator,
            version,
        })
        .then_ignore(ws)
        .separated_by(just(',').ignore_then(ws))
        .at_least(1);

    ws.ignore_then(ident)
        .then_ignore(ws)
        .then(
            ident
                .then_ignore(ws)
                .separated_by(just(',').ignore_then(ws))
                .at_least(1)
                .delimited_by(just('[').ignore_then(ws), just(']'))
                .then_ignore(ws)
                .or_else(|_| Ok(Vec::new())),
        )
        .then(
            just('@')
                .ignore_then(ws)
                .ignore_then(url::parser())
                .map(Spec::Url)
                .or(version_spec
                    .delimited_by(just('(').then_ignore(ws), just(')'))
                    .or(version_spec)
                    .map(Spec::Version))
                .then_ignore(ws)
                .or_not(),
        )
        .then(
            just(';')
                .ignore_then(ws)
                .ignore_then(recursive(|marker_or| {
                    macro_rules! c {
                        () => {
                            ' ' | '\t' | 'A' ..= 'Z' | 'a' ..= 'z' | '0' ..= '9' | '(' | ')' | '.' |
                            '{' | '}' | '-' | '_' | '*' | '#' | ':' | ';' | ',' | '/' | '?' | '[' |
                            ']' | '!' | '~' | '`' | '@' | '$' | '%' | '^' | '&' | '=' | '+' | '|' |
                            '<' | '>'
                        };
                    }

                    let marker_var = choice((
                        just('\'')
                            .ignore_then(set!(c!() | '"').repeated())
                            .then_ignore(just('\''))
                            .or(just('"')
                                .ignore_then(set!(c!() | '\'').repeated())
                                .then_ignore(just('"')))
                            .collect()
                            .map(Variable::String),
                        just("python_version").to(Variable::PythonVersion),
                        just("python_full_version").to(Variable::PythonFullVersion),
                        just("os_name").to(Variable::OsName),
                        just("sys_platform").to(Variable::SysPlatform),
                        just("platform_release").to(Variable::PlatformRelease),
                        just("platform_system").to(Variable::PlatformSystem),
                        just("platform_version").to(Variable::PlatformVersion),
                        just("platform_machine").to(Variable::PlatformMachine),
                        just("platform_python_implementation")
                            .to(Variable::PlatformPythonImplementation),
                        just("implementation_name").to(Variable::ImplementationName),
                        just("implementation_version").to(Variable::ImplementationVersion),
                        just("extra").to(Variable::Extra),
                    ));

                    let marker_expr = marker_var
                        .clone()
                        .then_ignore(ws)
                        .then(
                            cmp.map(Operator::Comparator)
                                .or(just("in").to(Operator::In).or(just("not")
                                    .ignore_then(set!(' ' | '\t').repeated().at_least(1))
                                    .ignore_then(just("in"))
                                    .to(Operator::NotIn))),
                        )
                        .then_ignore(ws)
                        .then(marker_var)
                        .map(|((lhs, op), rhs)| Marker::Operator(lhs, op, rhs))
                        .or(marker_or
                            .then_ignore(ws)
                            .delimited_by(just('(').then_ignore(ws), just(')')));

                    let marker_and = marker_expr
                        .clone()
                        .then(
                            ws.ignore_then(just("and"))
                                .ignore_then(ws)
                                .ignore_then(marker_expr)
                                .or_not(),
                        )
                        .map(|(lhs, rhs)| match rhs {
                            Some(rhs) => Marker::And(Box::new(lhs), Box::new(rhs)),
                            None => lhs,
                        });

                    marker_and
                        .clone()
                        .then(
                            ws.ignore_then(just("or"))
                                .ignore_then(ws)
                                .ignore_then(marker_and)
                                .or_not(),
                        )
                        .map(|(lhs, rhs)| match rhs {
                            Some(rhs) => Marker::Or(Box::new(lhs), Box::new(rhs)),
                            None => lhs,
                        })
                }))
                .or_not(),
        )
        .then_ignore(ws)
        .map(|(((name, extras), spec), marker)| Dependency {
            name,
            extras,
            spec,
            marker,
        })
}
