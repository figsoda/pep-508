macro_rules! set {
    ($($tt:tt)+) => {
        ::chumsky::primitive::filter(|c| matches!(c, $($tt)+))
    };
}

pub(crate) use set;
