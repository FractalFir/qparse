pub struct Octal;
pub struct Binary;
pub struct Display;
pub struct UpperHex;
pub struct LowerHex;
pub trait Parseable<T>: Sized {
    fn parse(input: &str) -> nom::IResult<&str, Self>;
}
// Int parser impls
mod int;

  
        impl crate::Parseable<crate::Display> for f32{
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::number::complete::float(input)
            }
        }
    
impl crate::Parseable<crate::Display> for f64{
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::number::complete::double(input)
            }
        }
