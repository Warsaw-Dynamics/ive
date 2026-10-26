// auto-generated: "lalrpop 0.23.2"
// sha3: 850d71284211d810756ae7e01deee6edcd44b69b022e4e434f3b0083b05f9c6a
// features: 
use crate::plantuml::{activity::{Block, Instruction, Branch, Condition, GroupKind, Join}, parser::{activity::Token, error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Activity {

    use crate::plantuml::{activity::{Block, Instruction, Branch, Condition, GroupKind, Join}, parser::{activity::Token, error::ParseFailure}};
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    #[allow(unused_extern_crates)]
    extern crate alloc;
    use super::__ToTriple;
    #[allow(dead_code)]
    pub(crate) enum __Symbol<>
     {
        Variant0(Token),
        Variant1(GroupKind),
        Variant2(String),
        Variant3(usize),
        Variant4(Block),
        Variant5((String, Block)),
        Variant6(Vec<Branch>),
        Variant7(Condition),
        Variant8(Instruction),
        Variant9(Join),
        Variant10(Vec<Block>),
        Variant11((Condition, String)),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -4, -4, -4, -4, 0, 0, 0, -4, 0, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 1
        45, 46, 42, 3, 0, 0, 0, 5, 0, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 52, 0, 0, -10,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
        // State 5
        -4, -4, -4, -4, 0, 0, 0, -4, 0, -4, 0, -4, -4, -4, -4, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -36, -36, 0, 0, 0, 0, -36, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, -24,
        // State 9
        45, 46, 42, 3, 0, 0, 0, 5, 0, 4, 0, 43, -30, -30, -30, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 10
        -4, -4, -4, -4, -4, -4, -4, -4, 0, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -12,
        // State 12
        -4, -4, -4, -4, 0, 0, 0, -4, 0, -4, -4, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 13
        -4, -4, -4, -4, 0, 0, 0, -4, -4, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -12,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 69, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        -4, -4, -4, -4, 0, 0, 0, -4, 0, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, -4, -4, 0, 0, 0,
        // State 17
        45, 46, 42, 3, -8, -8, -8, 5, 0, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -36, -36, 0, 0, 0, 0, -36, 0,
        // State 19
        45, 46, 42, 3, 0, 0, 0, 5, 0, 4, 25, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 20
        45, 46, 42, 3, 0, 0, 0, 5, 26, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 0, 0, 0, 0, -28,
        // State 22
        45, 46, 42, 3, 0, 0, 0, 5, 0, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 75, 41, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 29, 28, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -39,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -12,
        // State 26
        -4, -4, -4, -4, 0, 0, 0, -4, 0, -4, 0, -4, -4, -4, -4, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -12,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 33, -41, 0, 0, 0, 0, 0, 0, 0, -41,
        // State 30
        45, 46, 42, 3, 0, 0, 0, 5, 0, 4, 0, 43, -31, -31, -31, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, -33,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        -4, -4, -4, -4, 0, 0, -4, -4, 0, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 34
        -4, -4, -4, -4, -4, -4, -4, -4, 0, -4, 0, -4, 0, 0, 0, -4, 0, 0, 0, 0, 0, 0, 0, -4, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, -12,
        // State 36
        45, 46, 42, 3, 0, 0, -7, 5, 0, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 37
        45, 46, 42, 3, -9, -9, -9, 5, 0, 4, 0, 43, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 49, 0,
        // State 44
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0,
        // State 47
        -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, -17, 0, 0, 0, 0, 0, 0, -17, -17, 0, 0, 0,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 57, 0, 0, 0, 0, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11,
        // State 50
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13,
        // State 51
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0,
        // State 52
        -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, -15, 0, 0, 0, 0, 0, 0, -15, -15, 0, 0, 0,
        // State 53
        -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, -16, 0, 0, 0, 0, 0, 0, -16, -16, 0, 0, 0,
        // State 54
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14,
        // State 55
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62,
        // State 56
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19, 65, 0, 0, 0, 0, 66, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 67, 0, 0,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32,
        // State 61
        -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, -18, 0, 0, 0, 0, 0, 0, -18, -18, 0, 0, 0,
        // State 62
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44,
        // State 63
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -13,
        // State 64
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -35, -35, -35, 0, 0, 0, 0, 0, 0, 0, -35,
        // State 65
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -37, -37, 0, 0, 0, 0, -37, 0,
        // State 66
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11,
        // State 67
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25,
        // State 68
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, 0, 0, 0, 0, -26,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27, 0, 0, 0, 0, -27,
        // State 70
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27,
        // State 71
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19, 77, 0, 0, 0, 0, 66, 0,
        // State 72
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80,
        // State 73
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 81, 0,
        // State 74
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 82,
        // State 75
        0, 0, 0, 0, 0, 0, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -38, -38, 0, 0, 0, 0, -38, 0,
        // State 77
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 87,
        // State 79
        -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, -22, 0, 0, 0, 0, 0, 0, -22, -22, 0, 0, 0,
        // State 80
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0, 0, 0,
        // State 81
        -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, -23, 0, 0, 0, 0, 0, 0, -23, -23, 0, 0, 0,
        // State 82
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 89,
        // State 83
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34,
        // State 84
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35,
        // State 85
        -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, -21, 0, 0, 0, 0, 0, 0, -21, -21, 0, 0, 0,
        // State 86
        -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, -20, 0, 0, 0, 0, 0, 0, -20, -20, 0, 0, 0,
        // State 87
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29,
        // State 88
        -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, -19, 0, 0, 0, 0, 0, 0, -19, -19, 0, 0, 0,
        // State 89
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -40,
        // State 90
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -42, 0, 0, 0, 0, 0, 0, 0, -42,
        // State 91
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 27 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -4,
        // State 1
        -3,
        // State 2
        0,
        // State 3
        0,
        // State 4
        0,
        // State 5
        0,
        // State 6
        0,
        // State 7
        0,
        // State 8
        0,
        // State 9
        0,
        // State 10
        0,
        // State 11
        0,
        // State 12
        0,
        // State 13
        0,
        // State 14
        0,
        // State 15
        0,
        // State 16
        0,
        // State 17
        0,
        // State 18
        0,
        // State 19
        0,
        // State 20
        0,
        // State 21
        0,
        // State 22
        0,
        // State 23
        0,
        // State 24
        0,
        // State 25
        0,
        // State 26
        0,
        // State 27
        0,
        // State 28
        0,
        // State 29
        0,
        // State 30
        0,
        // State 31
        0,
        // State 32
        0,
        // State 33
        0,
        // State 34
        0,
        // State 35
        0,
        // State 36
        0,
        // State 37
        0,
        // State 38
        -45,
        // State 39
        -5,
        // State 40
        0,
        // State 41
        0,
        // State 42
        0,
        // State 43
        0,
        // State 44
        0,
        // State 45
        0,
        // State 46
        0,
        // State 47
        -17,
        // State 48
        0,
        // State 49
        0,
        // State 50
        0,
        // State 51
        0,
        // State 52
        -15,
        // State 53
        -16,
        // State 54
        0,
        // State 55
        0,
        // State 56
        0,
        // State 57
        0,
        // State 58
        0,
        // State 59
        0,
        // State 60
        0,
        // State 61
        -18,
        // State 62
        0,
        // State 63
        0,
        // State 64
        0,
        // State 65
        0,
        // State 66
        0,
        // State 67
        0,
        // State 68
        0,
        // State 69
        0,
        // State 70
        0,
        // State 71
        0,
        // State 72
        0,
        // State 73
        0,
        // State 74
        0,
        // State 75
        0,
        // State 76
        0,
        // State 77
        0,
        // State 78
        0,
        // State 79
        -22,
        // State 80
        0,
        // State 81
        -23,
        // State 82
        0,
        // State 83
        0,
        // State 84
        0,
        // State 85
        -21,
        // State 86
        -20,
        // State 87
        0,
        // State 88
        -19,
        // State 89
        0,
        // State 90
        0,
        // State 91
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 38,
            3 => match state {
                5 => 9,
                10 => 17,
                12 => 19,
                13 => 20,
                16 => 22,
                26 => 30,
                33 => 36,
                34 => 37,
                _ => 1,
            },
            4 => 75,
            5 => 23,
            6 => 50,
            7 => match state {
                14 => 67,
                25 => 78,
                27 => 83,
                35 => 91,
                _ => 62,
            },
            8 => match state {
                28 => 84,
                _ => 49,
            },
            9 => 39,
            10 => 60,
            11 => 21,
            12 => 72,
            13 => 15,
            14 => 54,
            15 => 89,
            16 => match state {
                2 | 28 => 6,
                4 => 8,
                24 => 29,
                32 => 90,
                _ => 63,
            },
            17 => match state {
                18 => 71,
                _ => 58,
            },
            18 => 77,
            19 => 31,
            20 => 57,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""start""###,
        r###""stop""###,
        r###""end""###,
        r###""if""###,
        r###""elseif""###,
        r###""else""###,
        r###""endif""###,
        r###""while""###,
        r###""endwhile""###,
        r###""repeat""###,
        r###""repeat while""###,
        r###""fork""###,
        r###""fork again""###,
        r###""end fork""###,
        r###""end merge""###,
        r###""group kind""###,
        r###""then""###,
        r###""is""###,
        r###""not""###,
        r###""(""###,
        r###"")""###,
        r###""{""###,
        r###""}""###,
        r###"":""###,
        r###"";""###,
        r###""text""###,
        r###""newline""###,
    ];
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            let next_state = __action(__state, index);
            if next_state == 0 {
                None
            } else {
                Some(alloc::string::ToString::to_string(terminal))
            }
        }).collect()
    }
    fn __expected_tokens_from_states<
        '__0,
        '__1,
    >(
        __states: &[i8],
        _: core::marker::PhantomData<()>,
    ) -> alloc::vec::Vec<alloc::string::String>
    where
        '__0: '__1,
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<()>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'__0, '__1>
    where '__0: '__1
    {
        builder: &'__1 mut crate::plantuml::parser::activity::Builder<'__0>,
        __phantom: core::marker::PhantomData<()>,
    }
    impl<'__0, '__1> __state_machine::ParserDefinition for __StateMachine<'__0, '__1>
    where '__0: '__1
    {
        type Location = usize;
        type Error = ParseFailure;
        type Token = Token;
        type TokenIndex = usize;
        type Symbol = __Symbol<>;
        type Success = Block;
        type StateIndex = i8;
        type Action = i8;
        type ReduceIndex = i8;
        type NonterminalIndex = usize;

        #[inline]
        fn start_location(&self) -> Self::Location {
              Default::default()
        }

        #[inline]
        fn start_state(&self) -> Self::StateIndex {
              0
        }

        #[inline]
        fn token_to_index(&self, token: &Self::Token) -> Option<usize> {
            __token_to_integer(token, core::marker::PhantomData::<()>)
        }

        #[inline]
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 27 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i8) -> i8 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i8, nt: usize) -> i8 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<()>)
        }

        fn expected_tokens(&self, state: i8) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens(state)
        }

        fn expected_tokens_from_states(&self, states: &[i8]) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens_from_states(states, core::marker::PhantomData::<()>)
        }

        #[inline]
        fn uses_error_recovery(&self) -> bool {
            false
        }

        #[inline]
        fn error_recovery_symbol(
            &self,
            recovery: __state_machine::ErrorRecovery<Self>,
        ) -> Self::Symbol {
            panic!("error recovery not enabled for this grammar")
        }

        fn reduce(
            &mut self,
            action: i8,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i8>,
            symbols: &mut alloc::vec::Vec<__state_machine::SymbolTriple<Self>>,
        ) -> Option<__state_machine::ParseResult<Self>> {
            __reduce(
                self.builder,
                action,
                start_location,
                states,
                symbols,
                core::marker::PhantomData::<()>,
            )
        }

        fn simulate_reduce(&self, action: i8) -> __state_machine::SimulatedReduce<Self> {
            __simulate_reduce(action, core::marker::PhantomData::<()>)
        }
    }
    fn __token_to_integer<
    >(
        __token: &Token,
        _: core::marker::PhantomData<()>,
    ) -> Option<usize>
    {
        #[warn(unused_variables)]
        match __token {
            Token::Start if true => Some(0),
            Token::Stop if true => Some(1),
            Token::End if true => Some(2),
            Token::If if true => Some(3),
            Token::ElseIf if true => Some(4),
            Token::Else if true => Some(5),
            Token::EndIf if true => Some(6),
            Token::While if true => Some(7),
            Token::EndWhile if true => Some(8),
            Token::Repeat if true => Some(9),
            Token::RepeatWhile if true => Some(10),
            Token::Fork if true => Some(11),
            Token::ForkAgain if true => Some(12),
            Token::EndFork if true => Some(13),
            Token::EndMerge if true => Some(14),
            Token::Group(_) if true => Some(15),
            Token::Then if true => Some(16),
            Token::Is if true => Some(17),
            Token::Not if true => Some(18),
            Token::Open if true => Some(19),
            Token::Close if true => Some(20),
            Token::BraceOpen if true => Some(21),
            Token::BraceClose if true => Some(22),
            Token::Colon if true => Some(23),
            Token::Semi if true => Some(24),
            Token::Text(_) if true => Some(25),
            Token::Newline if true => Some(26),
            _ => None,
        }
    }
    fn __token_to_symbol<
    >(
        __token_index: usize,
        __token: Token,
        _: core::marker::PhantomData<()>,
    ) -> __Symbol<>
    {
        #[allow(clippy::manual_range_patterns)]match __token_index {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 26 => __Symbol::Variant0(__token),
            15 => match __token {
                Token::Group(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            25 => match __token {
                Token::Text(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        '__0,
        '__1,
    >(
        __reduce_index: i8,
        _: core::marker::PhantomData<()>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'__0, '__1>>
    where
        '__0: '__1,
    {
        match __reduce_index {
            0 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 0,
                }
            }
            1 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 1,
                }
            }
            2 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 2,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 3,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 3,
                }
            }
            5 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 4,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 4,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 5,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 6,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 6,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 7,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 7,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 9,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 9,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 9,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 9,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 9,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 9,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 10,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 10,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            27 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 12,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            29 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 13,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 14,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 15,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 15,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 16,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 17,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 17,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 17,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 18,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 18,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 19,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 19,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 20,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 20,
                }
            }
            44 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct ActivityParser {
        _priv: (),
    }

    impl Default for ActivityParser { fn default() -> Self { Self::new() } }
    impl ActivityParser {
        pub fn new() -> ActivityParser {
            ActivityParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::parser::activity::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<Block, __lalrpop_util::ParseError<usize, Token, ParseFailure>>
        {
            let __tokens = __tokens0.into_iter();
            let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
            __state_machine::Parser::drive(
                __StateMachine {
                    builder,
                    __phantom: core::marker::PhantomData::<()>,
                },
                __tokens,
            )
        }
    }
    fn __accepts<
        '__0,
        '__1,
    >(
        __error_state: Option<i8>,
        __states: &[i8],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<()>,
    ) -> bool
    where
        '__0: '__1,
    {
        let mut __states = __states.to_vec();
        __states.extend(__error_state);
        loop {
            let mut __states_len = __states.len();
            let __top = __states[__states_len - 1];
            let __action = match __opt_integer {
                None => __EOF_ACTION[__top as usize],
                Some(__integer) => __action(__top, __integer),
            };
            if __action == 0 { return false; }
            if __action > 0 { return true; }
            let (__to_pop, __nt) = match __simulate_reduce(-(__action + 1), core::marker::PhantomData::<()>) {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop, nonterminal_produced
                } => (states_to_pop, nonterminal_produced),
                __state_machine::SimulatedReduce::Accept => return true,
            };
            __states_len -= __to_pop;
            __states.truncate(__states_len);
            let __top = __states[__states_len - 1];
            let __next_state = __goto(__top, __nt);
            __states.push(__next_state);
        }
    }
    fn __reduce<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> Option<Result<Block,__lalrpop_util::ParseError<usize, Token, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            1 => {
                __reduce1(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            2 => {
                // Activity = Block => ActionFn(1);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action1::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (1, 2)
            }
            3 => {
                __reduce3(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            4 => {
                // Block = Block, Instruction => ActionFn(3);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action3::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (2, 3)
            }
            5 => {
                __reduce5(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            6 => {
                __reduce6(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            7 => {
                __reduce7(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            8 => {
                // ElseIfs = ElseIfs, "elseif", IfCondition, "newline", Block => ActionFn(14);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant4(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant7(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action14::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (5, 5)
            }
            9 => {
                __reduce9(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            11 => {
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            12 => {
                // Guard = Parenthesized => ActionFn(47);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action47::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 7)
            }
            13 => {
                __reduce13(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            14 => {
                __reduce14(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            15 => {
                __reduce15(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            16 => {
                __reduce16(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            17 => {
                __reduce17(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            18 => {
                // Instruction = "if", IfCondition, "newline", Block, ElseIfs, Else, "endif", "newline" => ActionFn(8);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant5(__symbols);
                let __sym4 = __pop_Variant6(__symbols);
                let __sym3 = __pop_Variant4(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action8::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (8, 9)
            }
            19 => {
                __reduce19(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            20 => {
                __reduce20(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            21 => {
                // Instruction = "fork", "newline", Block, Lanes, Join, JoinLabel, "newline" => ActionFn(11);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant2(__symbols);
                let __sym4 = __pop_Variant9(__symbols);
                let __sym3 = __pop_Variant10(__symbols);
                let __sym2 = __pop_Variant4(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action11::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (7, 9)
            }
            22 => {
                __reduce22(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            23 => {
                __reduce23(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            24 => {
                __reduce24(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            25 => {
                __reduce25(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            26 => {
                __reduce26(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            27 => {
                __reduce27(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            28 => {
                // JoinLabel = "{", "text", "}" => ActionFn(48);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action48::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 12)
            }
            29 => {
                __reduce29(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            30 => {
                // Lanes = Lanes, "fork again", "newline", Block => ActionFn(18);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant4(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant10(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action18::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant10(__nt), __end));
                (4, 13)
            }
            31 => {
                __reduce31(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            32 => {
                __reduce32(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            33 => {
                __reduce33(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            34 => {
                __reduce34(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            35 => {
                __reduce35(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            36 => {
                // Parts = Parts, "text" => ActionFn(41);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action41::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (2, 17)
            }
            37 => {
                // Parts = Parts, "(", Parts, ")" => ActionFn(42);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action42::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 17)
            }
            38 => {
                __reduce38(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            39 => {
                __reduce39(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            40 => {
                __reduce40(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            41 => {
                __reduce41(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            42 => {
                __reduce42(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            43 => {
                __reduce43(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            44 => {
                // __Activity = Activity => ActionFn(0);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(builder, __sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {__action}")
        };
        let __states_len = __states.len();
        __states.truncate(__states_len - __pop_states);
        let __state = *__states.last().unwrap();
        let __next_state = __goto(__state, __nonterminal);
        __states.push(__next_state);
        None
    }
    #[inline(never)]
    fn __symbol_type_mismatch() -> ! {
        panic!("symbol type mismatch")
    }
    fn __pop_Variant11<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (Condition, String), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (String, Block), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Block, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Condition, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, GroupKind, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Instruction, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Join, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Token, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<Block>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<Branch>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(44);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action44::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(43);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action43::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 1)
    }
    fn __reduce3<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Block =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 3)
    }
    fn __reduce5<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Else =  => ActionFn(15);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action15::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 4)
    }
    fn __reduce6<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Else = "else", Guard, "newline", Block => ActionFn(16);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant4(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action16::<>(builder, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (4, 4)
    }
    fn __reduce7<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ElseIfs =  => ActionFn(13);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action13::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 5)
    }
    fn __reduce9<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Entry =  => ActionFn(23);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action23::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 6)
    }
    fn __reduce10<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Entry = ":", "text", ";" => ActionFn(24);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action24::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 6)
    }
    fn __reduce11<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Guard =  => ActionFn(31);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action31::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 7)
    }
    fn __reduce13<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // IfCondition = Parenthesized, ThenGuard => ActionFn(25);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action25::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 8)
    }
    fn __reduce14<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "start", "newline" => ActionFn(4);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action4::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 9)
    }
    fn __reduce15<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "stop", "newline" => ActionFn(5);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action5::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 9)
    }
    fn __reduce16<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "end", "newline" => ActionFn(6);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action6::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 9)
    }
    fn __reduce17<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = ":", "text", ";", "newline" => ActionFn(7);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action7::<>(builder, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (4, 9)
    }
    fn __reduce19<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "while", LoopCondition, "newline", Block, "endwhile", Guard, "newline" => ActionFn(9);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant0(__symbols);
        let __sym5 = __pop_Variant2(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant4(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action9::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (7, 9)
    }
    fn __reduce20<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "repeat", Entry, "newline", Block, "repeat while", RepeatCondition, "newline" => ActionFn(10);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant0(__symbols);
        let __sym5 = __pop_Variant11(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant4(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action10::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (7, 9)
    }
    fn __reduce22<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Instruction = "group kind", "text", "{", "newline", Block, "}", "newline" => ActionFn(12);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant0(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant4(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action12::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (7, 9)
    }
    fn __reduce23<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // IsGuard =  => ActionFn(29);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action29::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 10)
    }
    fn __reduce24<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // IsGuard = "is", Guard => ActionFn(30);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action30::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 10)
    }
    fn __reduce25<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Join = "end fork" => ActionFn(19);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action19::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 11)
    }
    fn __reduce26<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Join = "end merge" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 11)
    }
    fn __reduce27<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // JoinLabel =  => ActionFn(21);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action21::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 12)
    }
    fn __reduce29<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Lanes =  => ActionFn(17);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action17::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (0, 13)
    }
    fn __reduce31<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LoopCondition = Parenthesized, IsGuard => ActionFn(26);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action26::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 14)
    }
    fn __reduce32<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // OutGuard =  => ActionFn(37);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action37::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 15)
    }
    fn __reduce33<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // OutGuard = "not", Guard => ActionFn(38);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action38::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 15)
    }
    fn __reduce34<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Parenthesized = "(", Parts, ")" => ActionFn(39);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action39::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 16)
    }
    fn __reduce35<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Parts =  => ActionFn(40);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action40::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 17)
    }
    fn __reduce38<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RepeatCondition =  => ActionFn(33);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action33::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 18)
    }
    fn __reduce39<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RepeatCondition = Parenthesized, RepeatGuard, OutGuard => ActionFn(34);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action34::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (3, 18)
    }
    fn __reduce40<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RepeatGuard =  => ActionFn(35);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action35::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 19)
    }
    fn __reduce41<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RepeatGuard = "is", Parenthesized => ActionFn(36);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action36::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 19)
    }
    fn __reduce42<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ThenGuard =  => ActionFn(27);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action27::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 20)
    }
    fn __reduce43<
    >(
        builder: &mut crate::plantuml::parser::activity::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ThenGuard = "then", Guard => ActionFn(28);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action28::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 20)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Activity::ActivityParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Block, usize),
) -> Block
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, block, _): (usize, Block, usize),
) -> Result<Block,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.finish(block).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Block
{
    Vec::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, block, _): (usize, Block, usize),
    (_, instruction, _): (usize, Instruction, usize),
) -> Result<Block,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.append(block, instruction).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::Start
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::Stop
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::End
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::Action(value)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, condition, _): (usize, Condition, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
    (_, more, _): (usize, Vec<Branch>, usize),
    (_, otherwise, _): (usize, (String, Block), usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<Instruction,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.conditional(Branch { condition, body }, more, otherwise).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, condition, _): (usize, Condition, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
    (_, _, _): (usize, Token, usize),
    (_, out, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::While { condition, body, out }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, entry, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
    (_, _, _): (usize, Token, usize),
    (_, tail, _): (usize, (Condition, String), usize),
    (_, _, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::Repeat { entry, body, condition: tail.0, out: tail.1 }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, lane, _): (usize, Block, usize),
    (_, lanes, _): (usize, Vec<Block>, usize),
    (_, join, _): (usize, Join, usize),
    (_, label, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<Instruction,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.fork(lane, lanes, (join, label)).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, kind, _): (usize, GroupKind, usize),
    (_, name, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Instruction
{
    Instruction::Group { name, kind, body }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Branch>
{
    Vec::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, branches, _): (usize, Vec<Branch>, usize),
    (_, _, _): (usize, Token, usize),
    (_, condition, _): (usize, Condition, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
) -> Result<Vec<Branch>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.append(branches, Branch { condition, body }).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> (String, Block)
{
    (String::new(), Vec::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, guard, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
) -> (String, Block)
{
    (guard, body)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Block>
{
    Vec::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, lanes, _): (usize, Vec<Block>, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, body, _): (usize, Block, usize),
) -> Result<Vec<Block>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.append(lanes, body).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Join
{
    Join::Synchronize
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Join
{
    Join::Merge
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, text, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.join_label(text, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, text, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> String
{
    text
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, test, _): (usize, String, usize),
    (_, guard, _): (usize, String, usize),
) -> Condition
{
    Condition { test, guard }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, test, _): (usize, String, usize),
    (_, guard, _): (usize, String, usize),
) -> Condition
{
    Condition { test, guard }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action28<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, guard, _): (usize, String, usize),
) -> String
{
    guard
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action29<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action30<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, guard, _): (usize, String, usize),
) -> String
{
    guard
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action31<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action32<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.guard(value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action33<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> (Condition, String)
{
    (Condition { test: String::new(), guard: String::new() }, String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action34<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, test, _): (usize, String, usize),
    (_, guard, _): (usize, String, usize),
    (_, out, _): (usize, String, usize),
) -> (Condition, String)
{
    (Condition { test, guard }, out)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action35<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action36<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, guard, _): (usize, String, usize),
) -> String
{
    guard
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action37<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action38<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, guard, _): (usize, String, usize),
) -> String
{
    guard
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action39<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action40<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action41<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, value, _): (usize, String, usize),
    (_, text, _): (usize, String, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.concat(value, &text).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action42<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, nested, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.nested(value, nested).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action43<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action44<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action45<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action44(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action32(
        builder,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action46<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action44(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        builder,
        __temp0,
        __0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action47<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __0: (usize, String, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action43(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action45(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action48<
>(
    builder: &mut crate::plantuml::parser::activity::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action43(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action46(
        builder,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(clippy::type_complexity, dead_code)]
pub trait __ToTriple<>
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, ParseFailure>>;
}

impl<> __ToTriple<> for (usize, Token, usize)
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, ParseFailure>> {
        Ok(self)
    }
}
impl<> __ToTriple<> for Result<(usize, Token, usize), ParseFailure>
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, ParseFailure>> {
        self.map_err(|error| __lalrpop_util::ParseError::User { error })
    }
}
