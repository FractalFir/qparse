macro_rules! u{
    ($u:ident)=>{
        impl crate::Parseable<crate::Display> for $u{
            fn parse(input:&str)->nom::IResult<&str,Self>{
                nom::character::complete::$u(input)
            }
        }
    }
}
u!{u8}
u!{u16}
u!{u32}
u!{u64}
u!{u128}