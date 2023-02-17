use chumsky::{
    primitive::{choice, filter, just},
    Error, Parser,
};

use crate::macros::set;

macro_rules! c {
    () => {
        'A' ..= 'Z' | 'a' ..= 'z' | '0' ..= '9' | '-' | '.' | '_' | '~' |
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
    };
}

pub(crate) fn parser<E: Error<char> + 'static>() -> impl Parser<char, String, Error = E> {
    let c = set!(c!());
    let digit = filter(char::is_ascii_digit);
    let hex = filter(char::is_ascii_hexdigit);
    let percent = just('%').then_ignore(hex.repeated().exactly(2).rewind());
    let reg = percent.or(c);
    let pchar = percent.or(set!(c!() | ':' | '@'));

    let octet = choice((
        just('1').chain(digit).chain(digit),
        just('2').chain(
            set!('0' ..= '4')
                .chain(digit)
                .or(just('5').chain(set!('0' ..= '5'))),
        ),
        set!('1' ..= '9').chain(digit),
        digit.map(|c| vec![c]),
    ));
    let ipv4 = octet
        .chain(just('.'))
        .chain::<char, _, _>(octet)
        .chain(just('.'))
        .chain::<char, _, _>(octet)
        .chain(just('.'))
        .chain(octet);
    let h16 = hex.repeated().at_least(1).at_most(4);
    let h16r = h16.chain(just(':')).repeated();
    let ls32 = h16.chain(just(':')).chain(h16).or(ipv4);

    let segments = just('/').chain(pchar.repeated()).repeated().flatten();
    let frag = percent.or(set!(c!() | ':' | '@' | '/' | '?')).repeated();
    let frags = just('?')
        .chain(frag)
        .or_else(|_| Ok(Vec::new()))
        .chain::<char, _, _>(just('#').chain(frag).or_else(|_| Ok(Vec::new())));

    let path = just('/')
        .chain(
            just('/')
                .chain(
                    percent
                        .or(c)
                        .repeated()
                        .chain(just('@'))
                        .or_else(|_| Ok(Vec::new())),
                )
                .chain::<char, _, _>(choice((
                    reg.repeated(),
                    ipv4,
                    choice((
                        h16r.exactly(6).flatten().chain(ls32),
                        just(':')
                            .chain(just(':'))
                            .chain::<char, _, _>(h16r.exactly(5).flatten())
                            .chain(ls32),
                        h16.or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16r.exactly(4).flatten())
                            .chain(ls32),
                        h16.chain(just(':'))
                            .or_not()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16r.exactly(3).flatten())
                            .chain(ls32),
                        h16r.at_most(2)
                            .flatten()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16r.exactly(2).flatten())
                            .chain(ls32),
                        h16r.at_most(3)
                            .flatten()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16)
                            .chain(just(':'))
                            .chain(ls32),
                        h16r.at_most(4)
                            .flatten()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain(ls32),
                        h16r.at_most(5)
                            .flatten()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain(h16),
                        h16r.at_most(6)
                            .flatten()
                            .chain(h16)
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':')),
                        just('v')
                            .chain(hex.repeated().at_least(1))
                            .chain(just('.'))
                            .chain(set!(c!() | ':').repeated().at_least(1)),
                    ))
                    .delimited_by(just('['), just(']')),
                )))
                .chain::<char, _, _>(just(':').chain(digit.repeated()).or_not())
                .chain(segments)
                .or_not(),
        )
        .or_else(|_| Ok(Vec::new()));

    filter(char::is_ascii_alphabetic)
        .chain(set!('A' ..= 'Z' | 'a' ..= 'z' | '0' ..= '9' | '+' | '-' | '.').repeated())
        .chain(just(':'))
        .chain::<char, _, _>(path.or(pchar.repeated().at_least(1).chain(segments)))
        .or(path.or(percent
            .or(set!(c!() | '@'))
            .repeated()
            .at_least(1)
            .chain(segments)))
        .chain::<char, _, _>(frags)
        .collect()
}
