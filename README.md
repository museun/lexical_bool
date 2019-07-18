# lexical_bool

## LexicalBool provides a bool-like type that can be parsed from a string
```rust
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

### Using your own values
**note** This uses TLS, so the changes are only valid for the current thread
```rust
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
