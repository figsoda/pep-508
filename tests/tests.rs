use insta::assert_debug_snapshot;
use pep_508::parse;

#[test]
fn basic() {
    assert_debug_snapshot!(parse("requests"));
    assert_debug_snapshot!(parse("beautifulsoup4"));
    assert_debug_snapshot!(parse("Click>=7.0"));
    assert_debug_snapshot!(parse("tomli;python_version<'3.11'"));

    assert!(parse("").is_err());
    assert!(parse("# comment").is_err())
}

// tests from the PEP
#[test]
fn pep() {
    assert_debug_snapshot!(parse("A"));
    assert_debug_snapshot!(parse("A.B-C_D"));
    assert_debug_snapshot!(parse("aa"));
    assert_debug_snapshot!(parse("name"));
    assert_debug_snapshot!(parse("name<=1"));
    assert_debug_snapshot!(parse("name>=4"));
    assert_debug_snapshot!(parse("name>=3,<2"));
    assert_debug_snapshot!(parse("name@http://foo.com"));
    assert_debug_snapshot!(parse(
        "name [fred,bar] @ http://foo.com ; python_version=='3.7'"
    ));
    assert_debug_snapshot!(parse(
        "name[quux, strange];python_version<'3.7' and platform_version=='2'"
    ));
    assert_debug_snapshot!(parse("name; os_name=='a' or os_name=='b'"));
    assert_debug_snapshot!(parse("name; os_name=='a' and os_name=='b' or os_name=='c'"));
    assert_debug_snapshot!(parse(
        "name; os_name=='a' and (os_name=='b' or os_name=='c')"
    ));
    assert_debug_snapshot!(parse("name; os_name=='a' or os_name=='b' and os_name=='c'"));
    assert_debug_snapshot!(parse(
        "name; (os_name=='a' or os_name=='b') and os_name=='c'"
    ));
}
