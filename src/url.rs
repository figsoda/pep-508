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
    let h16r = just(':').chain(h16).repeated();
    let ls32 = h16.chain(just(':')).chain(h16).or(ipv4);

    let segments = just('/').chain(pchar.repeated()).repeated().flatten();
    let frag = percent.or(set!(c!() | ':' | '@' | '/' | '?')).repeated();
    let frags = just('?')
        .chain(frag)
        .or_else(|_| Ok(Vec::new()))
        .chain::<char, _, _>(just('#').chain(frag).or_else(|_| Ok(Vec::new())));

    let path = just('/').chain(
        just('/')
            .chain(
                percent
                    .or(set!(c!() | ':'))
                    .repeated()
                    .chain(just('@'))
                    .or_else(|_| Ok(Vec::new())),
            )
            .chain::<char, _, _>(choice((
                just('[')
                    .chain(choice((
                        just('v')
                            .chain(hex.repeated().at_least(1))
                            .chain(just('.'))
                            .chain(set!(c!() | ':').repeated().at_least(1)),
                        h16.chain(just(':'))
                            .repeated()
                            .exactly(6)
                            .flatten()
                            .chain(ls32),
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
                        h16.chain(h16r.at_most(2).flatten())
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16r.exactly(2).flatten())
                            .chain(ls32),
                        h16.chain(h16r.at_most(3).flatten())
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain::<char, _, _>(h16)
                            .chain(just(':'))
                            .chain(ls32),
                        h16.chain(h16r.at_most(4).flatten())
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain(ls32),
                        h16.chain(h16r.at_most(5).flatten())
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':'))
                            .chain(h16),
                        h16.chain(h16r.at_most(6).flatten())
                            .or_not()
                            .chain(just(':'))
                            .chain(just(':')),
                        // filter(|_| true).map(|_| todo!()),
                    )))
                    .chain(just(']')),
                ipv4,
                reg.repeated(),
            )))
            .chain::<char, _, _>(just(':').chain(digit.repeated()).or_not())
            .chain(segments)
            .or(pchar
                .repeated()
                .at_least(1)
                .chain(segments)
                .or_else(|_| Ok(Vec::new())))
            .or_not(),
    );

    filter(char::is_ascii_alphabetic)
        .chain(set!('A' ..= 'Z' | 'a' ..= 'z' | '0' ..= '9' | '+' | '-' | '.').repeated())
        .chain(just(':'))
        .chain::<char, _, _>(path.or(pchar.repeated().at_least(1).chain(segments)))
        .or(path)
        .or(percent
            .or(set!(c!() | '@'))
            .repeated()
            .at_least(1)
            .chain(segments))
        .chain::<char, _, _>(frags)
        .collect()
}

#[cfg(test)]
mod tests {
    use chumsky::{prelude::Simple, primitive::end, Parser};

    use super::parser;

    fn parse(s: &str) -> Result<String, Vec<Simple<char>>> {
        parser().then_ignore(end()).parse(s)
    }

    fn check(urls: impl IntoIterator<Item = impl AsRef<str>>) {
        for url in urls {
            let url = url.as_ref();
            assert_eq!(url, parse(url).expect(url));
        }
    }

    #[test]
    fn basic() {
        check([
            "https://github.com/figsoda/pep-508",
            "https://crates.io/search?q=pep-508&sort=recent-downloads",
            "http://127.0.0.1:8000?some=query#anchor",
            "/relative/url?query=good",
            "another/relative/url#this",
        ]);

        assert!(parse("").is_err());
        assert!(parse("https://example.com ").is_err());
    }

    // examples from https://datatracker.ietf.org/doc/html/rfc3986.htm
    #[test]
    fn examples() {
        check([
            "ftp://ftp.is.co.za/rfc/rfc1808.txt",
            "http://www.ietf.org/rfc/rfc2396.txt",
            "ldap://[2001:db8::7]/c=GB?objectClass?one",
            "mailto:John.Doe@example.com",
            "news:comp.infosystems.www.servers.unix",
            "tel:+1-816-555-1212",
            "telnet://192.0.2.16:80/",
            "urn:oasis:names:specification:docbook:dtd:xml:4.1.2",
        ]);
    }

    #[test]
    fn ipv6() {
        check([
            "https://[::]",
            "https://[::1]",
            "https://[0:0:0:0:0:0:0:0]",
            "https://[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]",
        ]);

        assert!(parse("[::]").is_err());
        assert!(parse("https://[:::1]").is_err());
        assert!(parse("https://[ffff:ffff:ffff:ffff:ffff:ffff:ffff]").is_err());
        assert!(parse("https://[0:0:0:0:0:0:0:0:0]").is_err());
    }
}
