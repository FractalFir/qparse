//! Lifted from the `std` docs:
//!
//! ```text
//! format_string := text [ maybe_format text ] *
//! maybe_format := '{' '{' | '}' '}' | format
//! format := '{' [ argument ] [ ':' format_spec ] [ ws ] * '}'
//! argument := integer | identifier
//!
//! format_spec := [[fill]align][sign]['#']['0'][width]['.' precision][type]
//! fill := character
//! align := '<' | '^' | '>'
//! sign := '+' | '-'
//! width := count
//! precision := count | '*'
//! type := '?' | 'x?' | 'X?' | 'o' | 'x' | 'X' | 'p' | 'b' | 'e' | 'E'
//! count := parameter | integer
//! parameter := argument
//! ```
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{take_till, take_while},
        tag,
    },
    character::complete::{anychar, char, multispace0, satisfy},
    combinator::{opt, recognize, success, value},
    multi::many0,
    sequence::{delimited, preceded},
};
#[derive(Debug)]
pub struct FormatString {
    pub text: String,
    pub fragments: Vec<(Format, String)>,
}
impl FormatString {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        nom::combinator::all_consuming(
            (text, many0((Format::parse, text)))
                .map(|(text, fragments)| FormatString { text, fragments }),
        )
        .parse(input)
    }
    /// Finds the named fields in this format string
    pub fn named_fields(&self) -> impl Iterator<Item = &Argument> + Clone {
        self.fragments.iter().map(|(f, _)| f.named_field())
    }
}
/// Parses the string part of a String format.
fn text(mut input: &str) -> IResult<&str, String> {
    let mut res = String::new();
    loop {
        // Take till the first format arg.
        let (reminder, chunk) = take_till(|c: char| c == '{' || c == '}').parse(input)?;
        res.push_str(chunk);
        input = reminder;
        // If this is an escaped '{' / '}' - continue on.
        if let Some(reminder) = input.strip_prefix("{{") {
            res.push('{');
            input = reminder;
        } else if let Some(reminder) = input.strip_prefix("}}") {
            res.push('}');
            input = reminder;
        }
        // If not - break. Responsibility of  the format arg parser.
        else {
            return Ok((reminder, res));
        }
    }
}
#[derive(Debug)]
pub(crate) struct Format {
    pub argument: Argument,
    pub fmt_spec: FormatSpec,
}
impl Format {
    fn parse(input: &str) -> IResult<&str, Self> {
        // format := '{' [ argument ] [ ':' format_spec ] [ ws ] * '}'
        // Per the rust grammar, this argument *should* be optional. `qparse`
        // handles types - field names are required.
        delimited(
            char('{'),
            (
                Argument::parse,
                opt(preceded(char(':'), FormatSpec::parse)),
                // Rust ignores whitespace here, noooot quite sure why.
                // mimicking this costs us nothing, so, hey.
                multispace0,
            ),
            char('}'),
        )
        .map(|(argument, fmt_spec, _)| Format {
            argument,
            fmt_spec: fmt_spec.unwrap_or_default(),
        })
        .parse(input)
    }

    fn named_field(&self) -> &Argument {
        &self.argument
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Align {
    Left,
    Center,
    Right,
}
impl Align {
    fn parse(input: &str) -> IResult<&str, Align> {
        alt((
            value(Align::Left, char('<')),
            value(Align::Center, char('^')),
            value(Align::Right, char('>')),
        ))
        .parse(input)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct FormatSpec {
    pub fill_align: Option<(Option<char>, Align)>,
    pub sign: Option<Sign>,
    pub alt_form: bool,
    pub sign_aware_zero_pad: bool,
    pub width: Option<Count>,
    pub precision: Option<Precision>,
    pub tpe: Type,
}
impl FormatSpec {
    fn parse(input: &str) -> IResult<&str, Self> {
        (
            opt(fill_align),
            opt(Sign::parse),
            opt(char('#')).map(|o| o.is_some()),
            opt(char('0')).map(|o| o.is_some()),
            opt(Count::parse),
            opt(preceded(char('.'), Precision::parse)),
            Type::parse,
        )
            .map(
                |(fill_align, sign, alt_form, sign_aware_zero_pad, width, precision, tpe)| Self {
                    fill_align,
                    sign,
                    alt_form,
                    sign_aware_zero_pad,
                    width,
                    precision,
                    tpe,
                },
            )
            .parse(input)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) enum Type {
    #[default]
    Display,
    Debug,
    DebugLowerHex,
    DebugUpperHex,
    Octal,
    LowerHex,
    UpperHex,
    Binary,
    UpperExp,
    LowerExp,
    Pointer,
}
impl Type {
    pub fn is_custom(&self) -> bool {
        false
    }
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Type::DebugLowerHex, tag("x?")),
            value(Type::DebugUpperHex, tag("X?")),
            value(Type::Debug, char('?')),
            value(Type::Octal, char('o')),
            value(Type::LowerHex, char('x')),
            value(Type::UpperHex, char('X')),
            value(Type::Pointer, char('p')),
            value(Type::Binary, char('b')),
            value(Type::LowerExp, char('e')),
            value(Type::UpperExp, char('E')),
            // Fallback
            success(Type::Display),
        ))
        .parse(input)
    }
}
#[derive(Clone, Debug)]
// Dead fields - fine, we just need to parse unsupported things correctly.
#[allow(dead_code)]
pub(crate) enum Precision {
    Count(Count),
    FromArg,
}
impl Precision {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            value(Precision::FromArg, char('*')),
            Count::parse.map(Precision::Count),
        ))
        .parse(input)
    }
}
fn fill_align(input: &str) -> IResult<&str, (Option<char>, Align)> {
    alt((
        (anychar, Align::parse).map(|(c, a)| (Some(c), a)),
        Align::parse.map(|a| (None, a)),
    ))
    .parse(input)
}

#[derive(Clone, Debug)]
// Dead fields - fine, we just need to parse unsupported things correctly.
#[allow(dead_code)]
pub(crate) enum Count {
    Parameter(Argument),
    Intiger(usize),
}
impl Count {
    fn parse(input: &str) -> IResult<&str, Count> {
        alt((
            parameter.map(Count::Parameter),
            nom::character::complete::usize.map(Count::Intiger),
        ))
        .parse(input)
    }
}
fn parameter(input: &str) -> IResult<&str, Argument> {
    (Argument::parse, char('$')).map(|(a, _)| a).parse(input)
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Argument {
    Intiger(usize),
    Identifier(String),
}
impl Argument {
    fn parse(input: &str) -> IResult<&str, Argument> {
        alt((
            identifier.map(Argument::Identifier),
            nom::character::complete::usize.map(Argument::Intiger),
        ))
        .parse(input)
    }

    /// Returns `true` if the argument is [`Intiger`].
    ///
    /// [`Intiger`]: Argument::Intiger
    #[must_use]
    pub(crate) fn is_intiger(&self) -> bool {
        matches!(self, Self::Intiger(..))
    }

    pub(crate) fn as_intiger(&self) -> Option<&usize> {
        if let Self::Intiger(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the argument is [`Identifier`].
    ///
    /// [`Identifier`]: Argument::Identifier
    #[must_use]
    pub(crate) fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier(..))
    }

    pub(crate) fn as_identifier(&self) -> Option<&String> {
        if let Self::Identifier(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum Sign {
    Plus,
    Minus,
}
impl Sign {
    fn parse(input: &str) -> IResult<&str, Sign> {
        alt((value(Sign::Plus, char('+')), value(Sign::Minus, char('-')))).parse(input)
    }
}
// Parser for rust identifiers, taken from rust reference.
fn identifier(input: &str) -> IResult<&str, String> {
    recognize((
        alt((
            satisfy(unicode_ident::is_xid_start),
            nom::character::char('_'),
        )),
        take_while(unicode_ident::is_xid_continue),
    ))
    .map(str::to_string)
    .parse(input)
}
#[test]
fn empty_fmt() {
    FormatString::parse("").unwrap();
}
#[test]
fn escped() {
    assert_eq!(FormatString::parse("{{").unwrap().1.text, "{");
    assert_eq!(FormatString::parse("}}").unwrap().1.text, "}");
    assert_eq!(FormatString::parse("{{}}").unwrap().1.text, "{}");
}

#[test]
fn fmt() {
    FormatString::parse("string{val:x}").unwrap();
}
