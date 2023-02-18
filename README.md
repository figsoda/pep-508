# pep-508

Rust implementation of Python dependency parser for PEP 508

[![version](https://img.shields.io/crates/v/pep-508?logo=rust&style=flat-square)](https://crates.io/crates/pep-508)
[![deps](https://deps.rs/repo/github/figsoda/pep-508/status.svg?style=flat-square&compact=true)](https://deps.rs/repo/github/figsoda/pep-508)
[![license](https://img.shields.io/badge/license-MPL--2.0-blue?style=flat-square)](https://www.mozilla.org/en-US/MPL/2.0)
[![ci](https://img.shields.io/github/actions/workflow/status/figsoda/pep-508/ci.yml?label=ci&logo=github-actions&style=flat-square)](https://github.com/<<github>>/actions/workflows/ci.yml)

[Documentation](https://docs.rs/pep-508)

## Usage

```rust
let dep = "requests[security, socks] <= 2.28.1, == 2.28.*; python_version > '3.7' and extra == 'http'";
let parsed = parse(dep).unwrap();
let expected = Dependency {
    name: "requests".to_owned(),
    extras: vec!["security".to_owned(), "socks".to_owned()],
    spec: Some(Spec::Version(vec![
        VersionSpec {
            comparator: Comparator::Le,
            version: "2.28.1".to_owned(),
        },
        VersionSpec {
            comparator: Comparator::Eq,
            version: "2.28.*".to_owned(),
        },
    ])),
    marker: Some(Marker::And(
        Box::new(Marker::Operator(
            Variable::PythonVersion,
            Operator::Comparator(Comparator::Gt),
            Variable::String("3.7".to_owned()),
        )),
        Box::new(Marker::Operator(
            Variable::Extra,
            Operator::Comparator(Comparator::Eq),
            Variable::String("http".to_owned()),
        )),
    )),
};
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md)
