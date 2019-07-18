/*!
# LexicalBool provides a bool-like type that can be parsed from a string
```rust
# use lexical_bool::LexicalBool;
let tests = &[
    // default `true` values
    ("true", true),
    ("t", true),
    ("1", true),
    ("yes", true),

    // default `false` values
    ("false", false),
    ("f", false),
    ("0", false),
    ("no", false),
];

for &(input, ok) in tests {
    let lb : LexicalBool = input.parse().expect("valid input");
    // LexicalBool can be "deref" into a bool, or compared directly with one (partialeq)
    assert_eq!(lb, ok);
}
```

## Using your own values
**note** This uses TLS, so the changes are only valid for the current thread
```rust
# use lexical_bool::LexicalBool;
// set the true values
assert!(lexical_bool::initialize_true_values(
    &[ "foo", "bar" ]
));
// set the false values
assert!(lexical_bool::initialize_false_values(
    &[ "baz", "qux" ]
));

// once set, you cannot change them (in this thread)
assert_eq!(lexical_bool::initialize_true_values(
    &[ "true", "1" ]
), false);

let tests = &[
    // your `true` values
    ("foo", true),
    ("bar", true),

    // your `false` values
    ("baz", false),
    ("qux", false),
];

for &(input, ok) in tests {
    // if parse (or from_str) is called before {initialize_true_values, initialize_false_values}
    // then it'll default to {lexical_bool::TRUTHY_VALUES, lexical_bool::FALSEY_VALUES}

    let lb : LexicalBool = input.parse().expect("valid input");
    // LexicalBool can be "deref" into a bool, or compared directly with one (partialeq)
    assert_eq!(lb, ok);
}

// ..and invalid bools
use std::str::FromStr as _;
use lexical_bool::Error;

let input = "true";
assert_eq!(
    LexicalBool::from_str(input),
    Err(Error::InvalidInput(input.to_string()))
);
```
*/

use once_cell::sync::OnceCell;

thread_local!(
    static TRUE_VALUES: OnceCell<Vec<String>> = OnceCell::new();
    static FALSE_VALUES: OnceCell<Vec<String>> = OnceCell::new();
);

/// Intialize a custom set of truth-y values
///
/// This returns whether or not they were updated
///
/// ***note*** They can only be updated once per thread
///
/// ***note*** If a parse happens in this thread and this hasn't be called, then it'll default to [`TRUTHY_VALUES`](./constant.TRUTHY_VALUES.html)
pub fn initialize_true_values<S: ToString>(values: impl IntoIterator<Item = S>) -> bool {
    let values = values.into_iter().map(|s| s.to_string()).collect();
    TRUE_VALUES.with(|f| f.set(values).is_ok())
}

/// Intialize a custom set of false-y values
///
/// This returns whether or not they were updated.
///
/// ***note*** They can only be updated once per thread
///
/// ***note*** If a parse happens in this thread and this hasn't be called, then it'll default to [`FALSEY_VALUES`](./constant.FALSEY_VALUES.html)
pub fn initialize_false_values<S: ToString>(values: impl IntoIterator<Item = S>) -> bool {
    let values = values.into_iter().map(|s| s.to_string()).collect();
    FALSE_VALUES.with(|f| f.set(values).is_ok())
}

/// `LexicalBool` allows parsing truthy-like strings to a bool
///
/// It can be `deref` (e.g. `*lb`) to get the bool, or compared to a bool (e.g. `lb == false`)
///
/// This uses the values provided by
/// [`initialize_true_values`](./fn.initialize_true_values.html) and
/// [`initialize_false_values`](./fn.initialize_false_values.html).
///
/// If they were
/// not initialized before the first parse, than the defaults of
/// [`TRUTHY_VALUES`](./constant.TRUTHY_VALUES.html) and
/// [`FALSEY_VALUES`](./constant.FALSEY_VALUES.html) are used for the life of
/// the thread it was parsed from.
///
/// ***note*** The parsing is case-insensitive
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct LexicalBool(bool);

impl std::ops::Deref for LexicalBool {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<bool> for LexicalBool {
    fn eq(&self, other: &bool) -> bool {
        *other == self.0
    }
}

impl std::str::FromStr for LexicalBool {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = s.to_ascii_lowercase();
        if TRUE_VALUES.with(|f| {
            f.get_or_init(|| TRUTHY_VALUES.iter().map(ToString::to_string).collect())
                .iter()
                .any(|k| k == &e)
        }) {
            return Ok(LexicalBool(true));
        }

        if FALSE_VALUES.with(|f| {
            f.get_or_init(|| FALSEY_VALUES.iter().map(ToString::to_string).collect())
                .iter()
                .any(|k| k == &e)
        }) {
            return Ok(LexicalBool(false));
        }

        Err(Error::InvalidInput(s.to_string()))
    }
}

/// Default truth-y values. Override with [`initialize_true_values`](./fn.initialize_true_values.html)
/// * `true`
/// * `t`
/// * `1`
/// * `yes`
pub const TRUTHY_VALUES: [&str; 4] = ["true", "t", "1", "yes"];

/// Default false-y values. Override with [`initialize_false_values`](./fn.initialize_false_values.html)
/// * `false`
/// * `f`
/// * `0`
/// * `no`
pub const FALSEY_VALUES: [&str; 4] = ["false", "f", "0", "no"];

/// An error returned by [`std::str::FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Invalid input while parsing the string
    InvalidInput(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput(input) => write!(
                f,
                "not a boolean: {}. only {} are allowed",
                input,
                TRUTHY_VALUES
                    .iter()
                    .chain(FALSEY_VALUES.iter())
                    .map(|val| format!("'{}''", val))
                    .fold(String::new(), |mut a, b| {
                        if !a.is_empty() {
                            a.push_str(", ")
                        }
                        a.push_str(&b);
                        a
                    })
            ),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_true() {
        let inputs = &[("true", true), ("t", true), ("1", true), ("yes", true)];
        for &(input, ok) in inputs {
            assert_eq!(input.parse::<LexicalBool>().unwrap(), ok);
        }
    }

    #[test]
    fn parse_false() {
        let inputs = &[("false", false), ("f", false), ("0", false), ("no", false)];
        for &(input, ok) in inputs {
            assert_eq!(input.parse::<LexicalBool>().unwrap(), ok);
        }
    }

    #[test]
    fn parse_custom_true() {
        assert!(initialize_true_values(&["this is true", "yep", "YEP"]));
        let inputs = &[
            ("this is true", true),
            ("yep", true),
            ("YEP", true),
            // keep the default false
            ("false", false),
            ("f", false),
            ("0", false),
            ("no", false),
        ];
        for &(input, ok) in inputs {
            assert_eq!(input.parse::<LexicalBool>().unwrap(), ok);
        }
    }

    #[test]
    fn parse_custom_false() {
        assert!(initialize_false_values(&["this is false", "nope", "NOPE"]));
        let inputs = &[
            ("this is false", false),
            ("nope", false),
            ("NOPE", false),
            // keep the default true
            ("true", true),
            ("t", true),
            ("1", true),
            ("yes", true),
        ];
        for &(input, ok) in inputs {
            assert_eq!(input.parse::<LexicalBool>().unwrap(), ok);
        }
    }
}
