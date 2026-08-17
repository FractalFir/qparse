use qparse::Parseable;
use qparse_macros::qparse;
#[derive(Debug, PartialEq)]
#[qparse("call @llvm.uwu.owo(i64 {some_val})")]
struct SomeInstr {
    some_val: u64,
}
#[qparse("{sat:present(.sat)}")]
struct DotSat {
    sat: bool,
}
#[derive(Debug, PartialEq)]
#[qparse("call @llvm.same_bitwidth(i{width} {some_val},i{width} {other_val})")]
struct VerifierTest {
    some_val: u64,
    width: u8,
    other_val: u64,
}
#[test]
fn some_instr_parse() {
    assert_eq!(
        SomeInstr::parse("call @llvm.uwu.owo(i64 67)").unwrap().1,
        SomeInstr { some_val: 67 }
    );
}
#[test]
fn verifier() {
    assert_eq!(
        VerifierTest::parse("call @llvm.same_bitwidth(i8 67,i8 123)")
            .unwrap()
            .1,
        VerifierTest {
            some_val: 67,
            other_val: 123,
            width: 8
        }
    );
    assert_eq!(
        VerifierTest::parse("call @llvm.same_bitwidth(i16 67,i16 123)")
            .unwrap()
            .1,
        VerifierTest {
            some_val: 67,
            other_val: 123,
            width: 16
        }
    );
    assert!(VerifierTest::parse("call @llvm.same_bitwidth(i16 67,i8 123)").is_err());
}
fn main() {
    println!(
        "{}",
        VerifierTest {
            some_val: 67,
            other_val: 123,
            width: 8
        }
    );
}
