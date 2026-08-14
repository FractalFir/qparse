pub struct Octal;
pub struct Binary;
pub struct Display;
pub trait Parseable<T>: Sized{
    fn parse(input:&str)->nom::IResult<&str,Self>;
}
// Int parser impls
mod int;