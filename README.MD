`qparse` - very simple parser library.
# What is qparse?

Writing parsers is... hard. Writing parsers AND formatters, which match(any parse result will roundtrip) is even harder.

`qparse` is a parsing macro library foccused on being very simple to use. The core idea is this: same syntax for parsing and formatting.

How do you parse a hex number in `qparse`?

```rust
#[qparse("0x{val:x}")]
struct HexNum{
   val:u64,
}
```

Exactly the same way you'd write a formatter in rust. QParse supports the rust fmt syntax AND extensions to it.

`x:present(.sat)` - checks if the _inner format match_ `.sat` is present.

QParse enum parsers are an alt over all the enum variants, in order.

```rust
#[qparse]
enum BobOrString{
   // All the strings starting with "bob" will be matched here!
   #[qparse("bob{0}")] 
   Bob(String),   
   #[qparse("{0}")]
   String(String),
}
```

