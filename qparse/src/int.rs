macro_rules! u {
    ($u:ident,$nonzero:ident) => {
        impl crate::Parseable<crate::Display> for $u {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                nom::character::complete::$u(input)
            }
        }
        impl crate::Parseable<crate::Octal> for $u {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_res(nom::character::complete::oct_digit1, |s: &str| {
                    <$u>::from_str_radix(s, 8)
                })
                .parse(input)
            }
        }
        impl crate::Parseable<crate::Binary> for $u {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_res(nom::character::complete::bin_digit1, |s: &str| {
                    <$u>::from_str_radix(s, 2)
                })
                .parse(input)
            }
        }
        impl crate::Parseable<crate::UpperHex> for $u {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_res(nom::character::complete::hex_digit1, |s: &str| {
                    <$u>::from_str_radix(s, 16)
                })
                .parse(input)
            }
        }
        impl crate::Parseable<crate::LowerHex> for $u {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_res(nom::character::complete::hex_digit1, |s: &str| {
                    <$u>::from_str_radix(s, 16)
                })
                .parse(input)
            }
        }
        impl crate::Parseable<crate::Display> for std::num::$nonzero {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_opt(
                    <$u as crate::Parseable<crate::Display>>::parse,
                    std::num::$nonzero::new,
                )
                .parse(input)
            }
        }
        impl crate::Parseable<crate::LowerHex> for std::num::$nonzero {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_opt(
                    <$u as crate::Parseable<crate::LowerHex>>::parse,
                    std::num::$nonzero::new,
                )
                .parse(input)
            }
        }
        impl crate::Parseable<crate::UpperHex> for std::num::$nonzero {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;
                nom::combinator::map_opt(
                    <$u as crate::Parseable<crate::LowerHex>>::parse,
                    std::num::$nonzero::new,
                )
                .parse(input)
            }
        }
        #[cfg(test)]
        mod $u {
            #[test]
            fn disp_0() {
                assert_eq!(
                    <$u as crate::Parseable<crate::Display>>::parse("0")
                        .unwrap()
                        .1,
                    0
                );
            }
            #[test]
            fn disp_max() {
                assert_eq!(
                    <$u as crate::Parseable<crate::Display>>::parse(&$u::MAX.to_string())
                        .unwrap()
                        .1,
                    $u::MAX
                );
            }
            #[test]
            fn octal_0() {
                assert_eq!(
                    <$u as crate::Parseable<crate::Octal>>::parse("0")
                        .unwrap()
                        .1,
                    0
                );
            }
            #[test]
            fn octal_9_is_err() {
                assert!(<$u as crate::Parseable<crate::Octal>>::parse("9").is_err());
            }
            #[test]
            fn bin_0() {
                assert_eq!(
                    <$u as crate::Parseable<crate::Binary>>::parse("0")
                        .unwrap()
                        .1,
                    0
                );
            }
            #[test]
            fn bin_67() {
                assert_eq!(
                    <$u as crate::Parseable<crate::Binary>>::parse("1000011")
                        .unwrap()
                        .1,
                    67
                );
            }
            #[test]
            fn hex_0() {
                assert_eq!(
                    <$u as crate::Parseable<crate::LowerHex>>::parse("0")
                        .unwrap()
                        .1,
                    0
                );
                assert_eq!(
                    <$u as crate::Parseable<crate::UpperHex>>::parse("0")
                        .unwrap()
                        .1,
                    0
                );
            }
        }
    };
}
macro_rules! i {
    ($i:ident) => {
        impl crate::Parseable<crate::Display> for $i {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                nom::character::complete::$i(input)
            }
        }
    };
}
u! {u8,NonZeroU8}
u! {u16,NonZeroU16}
u! {u32,NonZeroU32}
u! {u64,NonZeroU64}
u! {u128,NonZeroU128}
i! {i8}
i! {i16}
i! {i32}
i! {i64}
i! {i128}
