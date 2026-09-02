use nom::Parser;

pub struct Octal;
pub struct Binary;
pub struct Display;
pub struct UpperHex;
pub struct LowerHex;
pub trait Parseable<T>: Sized {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError>;
    fn simple_parse(input:&str)  -> nom::IResult<&str, Self>{
        Self::parse::<nom::error::Error<&str>>(input)
    }
}
// Int parser impls
mod int;

impl crate::Parseable<crate::Display> for f32 {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError> {
        use nom::Parser;
        nom::number::complete::float(input)
    }
}

impl crate::Parseable<crate::Display> for f64 {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError> {
        use nom::Parser;
        nom::number::complete::double(input)
    }
}

impl<T: Parseable<P>, P> crate::Parseable<P> for Box<T> {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError> {
        <T as crate::Parseable<P>>::parse
            .map(|t| Box::new(t))
            .parse(input)
    }
}
impl Parseable<Display> for bool {
    fn parse<'a, E>(input: &'a str) -> nom::IResult<&'a str, Self, E>
    where
        E: nom::error::ParseError<&'a str>+ nom::error::FromExternalError<&'a str, std::num::ParseIntError> {
        nom::branch::alt((
            nom::combinator::value(true, nom::bytes::complete::tag("true")),
            nom::combinator::value(false, nom::bytes::complete::tag("false")),
        ))
        .parse(input)
    }
}
