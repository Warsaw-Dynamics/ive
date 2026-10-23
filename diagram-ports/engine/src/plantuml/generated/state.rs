// auto-generated: "lalrpop 0.23.2"
// sha3: 31ced6d92e3dcbb035fc4ab261bfcacf60c14507be6f1ec057c67cf315d4b236
// features: 
use crate::plantuml::{state::{Axis, Caption, Declaration, Endpoint, Kind, Name, NamePath}, parser::{state::{Token, join_shaft, require_adjacent}, error::ParseFailure}};
use std::ops::Range;
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__State {

    use crate::plantuml::{state::{Axis, Caption, Declaration, Endpoint, Kind, Name, NamePath}, parser::{state::{Token, join_shaft, require_adjacent}, error::ParseFailure}};
    use std::ops::Range;
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
        Variant1(Name),
        Variant2(String),
        Variant3(Kind),
        Variant4(usize),
        Variant5(bool),
        Variant6(Range<usize>),
        Variant7(()),
        Variant8(Declaration),
        Variant9(Caption),
        Variant10(Endpoint),
        Variant11(NamePath),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -31, 0, -31, -31, -31, -31, -31, 0, -31, -31, 0, 0, -31, 0, 0, 0, 0, 0, 0, 0, -31, 0, 0,
        // State 1
        33, 0, 27, 28, 29, 30, 32, 0, 26, 31, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0,
        // State 2
        -31, 0, -31, -31, -31, -31, -31, 0, -31, -31, 0, 0, -31, 0, 0, -31, 0, 0, 0, 0, -31, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 25, 0, 0,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, -45, 0, 0, 0, -45, 0, 0, 41, 42, -45, 0, -45, 0, 0,
        // State 5
        33, 0, 27, 28, 45, 46, 32, 0, 26, 31, 0, 0, 34, 0, 0, 47, 0, 0, 0, 0, 25, 0, 0,
        // State 6
        12, 0, 27, 28, 29, 30, 32, 0, 51, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0,
        // State 8
        12, 0, 27, 28, 29, 30, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        12, 0, 27, 28, 29, 30, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, -33,
        // State 11
        0, 63, -21, -21, 0, 0, 0, 0, 0, -21, -21, -21, 0, -21, -21, 0, -21, -21, -21, 0, -21, 0, -21,
        // State 12
        0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 67, 0, -55, -55, 0, 0, 0, 0, -55,
        // State 13
        0, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21,
        // State 14
        12, 0, 27, 28, 29, 30, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 42, 0, 0, 0, 0, -15,
        // State 16
        12, 0, 27, 28, 29, 30, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        12, 0, 27, 28, 29, 30, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 38,
        // State 21
        -32, 0, -32, -32, -32, -32, -32, 0, -32, -32, 0, 0, -32, 0, 0, -32, 0, 0, 0, 0, -32, 0, 0,
        // State 22
        0, 0, 0, 0, 0, 0, 0, 0, 0, 43, 0, 0, 0, 9, 0, 0, 0, 0, -35, 0, -35, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 0, 0, 0, -39, 0, 0, 0, -39, 0, 0, -39, -39, -39, 0, -39, 0, 0,
        // State 24
        -23, 0, -23, -23, -23, -23, -23, 0, -23, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23, 0, -23,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -36, 0, -36, 0, 0,
        // State 26
        0, 0, -50, -50, 0, 0, 0, 0, 0, -50, -50, -50, 0, -50, -50, 0, -50, -50, -50, 0, -50, 0, -50,
        // State 27
        0, 0, -51, -51, 0, 0, 0, 0, 0, -51, -51, -51, 0, -51, -51, 0, -51, -51, -51, 0, -51, 0, -51,
        // State 28
        0, 0, -52, -52, 0, 0, 0, 0, 0, -52, -52, -52, 0, -52, -52, 0, -52, -52, -52, 0, -52, 0, -52,
        // State 29
        0, 0, -53, -53, 0, 0, 0, 0, 0, -53, -53, -53, 0, -53, -53, 0, -53, -53, -53, 0, -53, 0, -53,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -37, 0, -37, 0, 0,
        // State 31
        0, 0, -49, -49, 0, 0, 0, 0, 0, -49, -49, -49, 0, -49, -49, 0, -49, -49, -49, 0, -49, 0, -49,
        // State 32
        0, 10, 0, 0, 0, 0, 0, 0, 0, -40, 0, 0, 0, -40, 0, 0, -40, -40, -40, 0, -40, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5,
        // State 34
        -30, 0, -30, -30, -30, -30, -30, 0, -30, -30, 0, 0, -30, 0, 0, -30, 0, 0, 0, 0, -30, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6,
        // State 36
        -24, 0, -24, -24, -24, -24, -24, 0, -24, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -24, 0, -24,
        // State 37
        -29, 0, -29, -29, -29, -29, -29, 0, -29, -29, 0, 0, -29, 0, 0, -29, 0, 0, 0, 0, -29, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 37, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, -13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, -14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -38, 0, -38, 0, 0,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58,
        // State 44
        14, 0, 0, 0, 0, 0, 0, 0, 0, -52, 0, 0, 0, -52, 0, 0, -52, -52, -52, 0, -52, 0, 0,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, -53, 0, 0, 0, -53, 0, 0, -53, -53, -53, 0, -53, 0, -8,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, -47, 0, 0, 0, -47, 0, 0, -47, 0, 0, 0, 0, 0, -47,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 0, 61, 0, 0, 0, 15, 0, 0, -17, 0, 0, 0, 0, 0, -17,
        // State 49
        0, 0, -41, -41, 0, 0, 0, 0, 0, -41, -41, -41, 0, -41, -41, 0, -41, -41, -41, 0, -41, 0, -41,
        // State 50
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, 0, 0, 0, 0, -18,
        // State 51
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, 0, 0, 0, 0, -19,
        // State 52
        -3, 0, -3, -3, -3, -3, -3, 0, -3, -3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 53
        -4, 0, -4, -4, -4, -4, -4, 0, -4, -4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0,
        // State 54
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64,
        // State 55
        0, 0, 0, 0, 0, 0, 0, 0, 0, -46, 0, 0, 0, -46, 0, 0, 0, 0, -46, 0, -46, 0, 0,
        // State 56
        0, 0, 18, -11, 0, 0, 0, 0, 0, 0, -11, 0, 0, 0, -11, 0, -11, -11, 0, 0, 0, 0, -11,
        // State 57
        -26, 0, -26, -26, -26, -26, -26, 0, -26, -26, 0, 0, -26, 0, 0, -26, 0, 0, 0, 0, -26, 0, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 69,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, 0, 0, 0, -20,
        // State 61
        0, 0, -42, -42, 0, 0, 0, 0, 0, -42, -42, -42, 0, -42, -42, 0, -42, -42, -42, 0, -42, 0, -42,
        // State 62
        0, 0, -22, -22, 0, 0, 0, 0, 0, -22, -22, -22, 0, -22, -22, 0, -22, -22, -22, 0, -22, 0, -22,
        // State 63
        -27, 0, -27, -27, -27, -27, -27, 0, -27, -27, 0, 0, -27, 0, 0, -27, 0, 0, 0, 0, -27, 0, 0,
        // State 64
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 72,
        // State 65
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44,
        // State 66
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43,
        // State 67
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -9,
        // State 68
        -28, 0, -28, -28, -28, -28, -28, 0, -28, -28, 0, 0, -28, 0, 0, -28, 0, 0, 0, 0, -28, 0, 0,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34,
        // State 70
        0, 0, 0, 0, 0, 0, 0, 0, 0, -48, 0, 0, 0, -48, 0, 0, -48, 0, 0, 0, 0, 0, -48,
        // State 71
        -10, 0, -10, -10, -10, -10, -10, 0, -10, -10, 0, 0, -10, 0, 0, -10, 0, 0, 0, 0, -10, 0, 0,
        // State 72
        0, 0, 0, 0, 0, 0, 0, 77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 73
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 78,
        // State 74
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 75
        0, 0, 0, -12, 0, 0, 0, 0, 0, 0, -12, 0, 0, 0, -12, 0, -12, -12, 0, 0, 0, 0, -12,
        // State 76
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16,
        // State 77
        -25, 0, -25, -25, -25, -25, -25, 0, -25, -25, 0, 0, -25, 0, 0, -25, 0, 0, 0, 0, -25, 0, 0,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, -56, 0, 0, 0, 0, -56,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 23 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -31,
        // State 1
        -54,
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
        -57,
        // State 19
        0,
        // State 20
        0,
        // State 21
        -32,
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
        -30,
        // State 35
        0,
        // State 36
        0,
        // State 37
        -29,
        // State 38
        0,
        // State 39
        0,
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
        0,
        // State 48
        0,
        // State 49
        0,
        // State 50
        0,
        // State 51
        0,
        // State 52
        0,
        // State 53
        0,
        // State 54
        0,
        // State 55
        0,
        // State 56
        0,
        // State 57
        -26,
        // State 58
        0,
        // State 59
        0,
        // State 60
        0,
        // State 61
        0,
        // State 62
        0,
        // State 63
        -27,
        // State 64
        0,
        // State 65
        0,
        // State 66
        0,
        // State 67
        0,
        // State 68
        -28,
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
        -25,
        // State 78
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 6,
            3 => 19,
            4 => 43,
            5 => 2,
            6 => 12,
            7 => match state {
                15 => 72,
                _ => 39,
            },
            8 => 73,
            9 => 10,
            10 => match state {
                13 => 67,
                _ => 61,
            },
            11 => match state {
                3 => 38,
                7 => 53,
                _ => 20,
            },
            12 => 21,
            13 => match state {
                2 => 5,
                _ => 1,
            },
            14 => 58,
            15 => 3,
            16 => 4,
            17 => match state {
                8 => 55,
                9 => 56,
                14 => 70,
                16 => 74,
                17 => 75,
                _ => 47,
            },
            18 => 64,
            19 => 22,
            20 => 48,
            21 => match state {
                1 | 5 => 23,
                _ => 49,
            },
            22 => 18,
            23 => 15,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""state""###,
        r###""gap""###,
        r###""as""###,
        r###""begin""###,
        r###""end""###,
        r###""endstate""###,
        r###""name""###,
        r###""label""###,
        r###""[*]""###,
        r###""history""###,
        r###""<<""###,
        r###"">>""###,
        r###""|""###,
        r###"".""###,
        r###""{""###,
        r###""}""###,
        r###"":""###,
        r###""attached colon""###,
        r###""<""###,
        r###"">""###,
        r###""-""###,
        r###""unknown""###,
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
        builder: &'__1 mut crate::plantuml::state::Builder<'__0>,
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
        type Success = ();
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
            __action(state, 23 - 1)
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
            Token::State if true => Some(0),
            Token::Gap if true => Some(1),
            Token::As if true => Some(2),
            Token::Begin if true => Some(3),
            Token::End if true => Some(4),
            Token::EndState if true => Some(5),
            Token::Name(_) if true => Some(6),
            Token::Label(_) if true => Some(7),
            Token::Pseudo if true => Some(8),
            Token::History(_) if true => Some(9),
            Token::StereoOpen if true => Some(10),
            Token::StereoClose if true => Some(11),
            Token::Bar(_) if true => Some(12),
            Token::Dot if true => Some(13),
            Token::Open if true => Some(14),
            Token::Close if true => Some(15),
            Token::Colon if true => Some(16),
            Token::AttachedColon if true => Some(17),
            Token::Less(_) if true => Some(18),
            Token::Greater(_) if true => Some(19),
            Token::Minus(_) if true => Some(20),
            Token::Unknown if true => Some(21),
            Token::Newline if true => Some(22),
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
            0 | 1 | 2 | 3 | 4 | 5 | 8 | 10 | 11 | 13 | 14 | 15 | 16 | 17 | 21 | 22 => __Symbol::Variant0(__token),
            6 => match __token {
                Token::Name(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            7 => match __token {
                Token::Label(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            9 => match __token {
                Token::History(__tok0) if true => __Symbol::Variant3(__tok0),
                _ => unreachable!(),
            },
            12 | 18 | 19 | 20 => match __token {
                Token::Bar(__tok0) | Token::Less(__tok0) | Token::Greater(__tok0) | Token::Minus(__tok0) if true => __Symbol::Variant4(__tok0),
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
                    states_to_pop: 2,
                    nonterminal_produced: 2,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 2,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 3,
                }
            }
            5 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 3,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 4,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 4,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 4,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 5,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 6,
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
                    states_to_pop: 1,
                    nonterminal_produced: 7,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 8,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 10,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 11,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            27 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 12,
                }
            }
            29 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 12,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 13,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 14,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 14,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 15,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 16,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 16,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 17,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 17,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 18,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 18,
                }
            }
            44 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 19,
                }
            }
            45 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 19,
                }
            }
            46 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 20,
                }
            }
            47 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 20,
                }
            }
            48 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            49 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            50 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            51 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            52 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            53 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 22,
                }
            }
            54 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 23,
                }
            }
            55 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 23,
                }
            }
            56 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct StateParser {
        _priv: (),
    }

    impl Default for StateParser { fn default() -> Self { Self::new() } }
    impl StateParser {
        pub fn new() -> StateParser {
            StateParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::state::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<(), __lalrpop_util::ParseError<usize, Token, ParseFailure>>
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
        builder: &mut crate::plantuml::state::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> Option<Result<(),__lalrpop_util::ParseError<usize, Token, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            1 => {
                __reduce1(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            2 => {
                // Arrow = Hyphens, ">" => ActionFn(75);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant4(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action75::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 2)
            }
            3 => {
                // Arrow = "<", Hyphens => ActionFn(76);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant6(__symbols);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action76::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 2)
            }
            4 => {
                __reduce4(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            5 => {
                // Bars = Bars, "|" => ActionFn(77);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant4(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action77::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (2, 3)
            }
            6 => {
                __reduce6(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            7 => {
                __reduce7(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            8 => {
                __reduce8(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            9 => {
                // CompositeOpen = "state", "gap", Declaration, Opener, "newline" => ActionFn(78);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant7(__symbols);
                let __sym2 = __pop_Variant8(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action78::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (5, 5)
            }
            10 => {
                // Declaration = Name => ActionFn(18);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action18::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (1, 6)
            }
            11 => {
                // Declaration = Name, "as", Name => ActionFn(79);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action79::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (3, 6)
            }
            12 => {
                __reduce12(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                __reduce18(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            19 => {
                // Endpoint = Path<Name>, "history" => ActionFn(80);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant11(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action80::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant10(__nt), __end));
                (2, 9)
            }
            20 => {
                __reduce20(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            21 => {
                __reduce21(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            22 => {
                __reduce22(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            23 => {
                // Hyphens = Hyphens, "-" => ActionFn(81);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant4(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action81::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (2, 11)
            }
            24 => {
                // Item = "state", "gap", Declaration, Stereotype, DescriptionLabel, "newline" => ActionFn(82);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant9(__symbols);
                let __sym3 = __pop_Variant3(__symbols);
                let __sym2 = __pop_Variant8(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action82::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (6, 12)
            }
            25 => {
                // Item = CompositeOpen, Items, Closer, "newline" => ActionFn(5);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant7(__symbols);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant7(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action5::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (4, 12)
            }
            26 => {
                // Item = LeftName, DescriptionColon, "label", "newline" => ActionFn(83);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action83::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (4, 12)
            }
            27 => {
                // Item = LeftEndpoint, Arrow, Endpoint, Label, "newline" => ActionFn(84);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant9(__symbols);
                let __sym2 = __pop_Variant10(__symbols);
                let __sym1 = __pop_Variant5(__symbols);
                let __sym0 = __pop_Variant10(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action84::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (5, 12)
            }
            28 => {
                // Item = Hyphens, "newline" => ActionFn(85);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action85::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (2, 12)
            }
            29 => {
                // Item = Bars, "newline" => ActionFn(86);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action86::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (2, 12)
            }
            30 => {
                __reduce30(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            31 => {
                // Items = Items, Item => ActionFn(3);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant7(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action3::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (2, 13)
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
                __reduce36(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            37 => {
                // LeftEndpoint = Path<LeftName>, "history" => ActionFn(87);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant11(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action87::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant10(__nt), __end));
                (2, 15)
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
                // Path<LeftName> = LeftName => ActionFn(70);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action70::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant11(__nt), __end));
                (1, 19)
            }
            45 => {
                // Path<LeftName> = Path<LeftName>, ".", Name => ActionFn(71);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant11(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action71::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant11(__nt), __end));
                (3, 19)
            }
            46 => {
                // Path<Name> = Name => ActionFn(72);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action72::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant11(__nt), __end));
                (1, 20)
            }
            47 => {
                // Path<Name> = Path<Name>, ".", Name => ActionFn(73);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant11(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action73::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant11(__nt), __end));
                (3, 20)
            }
            48 => {
                __reduce48(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            49 => {
                __reduce49(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            50 => {
                __reduce50(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            51 => {
                __reduce51(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            52 => {
                __reduce52(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            53 => {
                __reduce53(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            54 => {
                __reduce54(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            55 => {
                // Stereotype = "<<", Name, ">>" => ActionFn(88);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action88::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 23)
            }
            56 => {
                // __State = State => ActionFn(0);
                let __sym0 = __pop_Variant7(__symbols);
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
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Caption, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Declaration, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Endpoint, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Kind, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Name, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant11<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, NamePath, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Range<usize>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
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
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, bool, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(56);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action56::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(55);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action55::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 1)
    }
    fn __reduce4<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Bars = "|" => ActionFn(39);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action39::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 3)
    }
    fn __reduce6<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Closer = "}" => ActionFn(13);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action13::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 4)
    }
    fn __reduce7<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Closer = "endstate" => ActionFn(14);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action14::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 4)
    }
    fn __reduce8<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Closer = "end", "state", Gap => ActionFn(15);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action15::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (3, 4)
    }
    fn __reduce12<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DescriptionColon = ":" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 7)
    }
    fn __reduce13<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DescriptionColon = "attached colon" => ActionFn(46);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action46::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 7)
    }
    fn __reduce14<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DescriptionLabel =  => ActionFn(43);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action43::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 8)
    }
    fn __reduce15<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DescriptionLabel = DescriptionColon, "label" => ActionFn(44);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action44::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (2, 8)
    }
    fn __reduce16<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Endpoint = Path<Name> => ActionFn(33);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action33::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 9)
    }
    fn __reduce17<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Endpoint = "[*]" => ActionFn(34);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action34::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 9)
    }
    fn __reduce18<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Endpoint = "history" => ActionFn(35);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action35::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 9)
    }
    fn __reduce20<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Gap =  => ActionFn(16);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action16::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (0, 10)
    }
    fn __reduce21<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Gap = "gap" => ActionFn(17);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
    }
    fn __reduce22<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Hyphens = "-" => ActionFn(49);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action49::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 11)
    }
    fn __reduce30<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Items =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (0, 13)
    }
    fn __reduce32<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label =  => ActionFn(41);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action41::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 14)
    }
    fn __reduce33<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label = ":", "label" => ActionFn(42);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action42::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (2, 14)
    }
    fn __reduce34<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LeftEndpoint = Path<LeftName> => ActionFn(29);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action29::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce35<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LeftEndpoint = "[*]" => ActionFn(30);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce36<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LeftEndpoint = "history" => ActionFn(31);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action31::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce38<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LeftName = PlainName => ActionFn(22);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action22::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 16)
    }
    fn __reduce39<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // LeftName = "state" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 16)
    }
    fn __reduce40<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Name = PlainName => ActionFn(20);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 17)
    }
    fn __reduce41<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Name = "state", Gap => ActionFn(21);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action21::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 17)
    }
    fn __reduce42<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Opener = "{" => ActionFn(11);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 18)
    }
    fn __reduce43<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Opener = "begin" => ActionFn(12);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 18)
    }
    fn __reduce48<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // PlainName = "name" => ActionFn(24);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action24::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 21)
    }
    fn __reduce49<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // PlainName = "as" => ActionFn(25);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action25::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 21)
    }
    fn __reduce50<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // PlainName = "begin" => ActionFn(26);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 21)
    }
    fn __reduce51<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // PlainName = "end" => ActionFn(27);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action27::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 21)
    }
    fn __reduce52<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // PlainName = "endstate" => ActionFn(28);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action28::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 21)
    }
    fn __reduce53<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // State = Items => ActionFn(1);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 22)
    }
    fn __reduce54<
    >(
        builder: &mut crate::plantuml::state::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Stereotype =  => ActionFn(37);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action37::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 23)
    }
}
#[allow(unused_imports)]
pub use self::__parse__State::StateParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.tick().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, Declaration, usize),
    (_, kind, _): (usize, Kind, usize),
    (_, label, _): (usize, Caption, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.declare((value, kind, label), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, _open, _): (usize, (), usize),
    (_, _items, _): (usize, (), usize),
    (_, _closer, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
    (_, _colon, _): (usize, (), usize),
    (_, label, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.describe((name, label), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, left, _): (usize, Endpoint, usize),
    (_, reverse, _): (usize, bool, usize),
    (_, right, _): (usize, Endpoint, usize),
    (_, label, _): (usize, Caption, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.transition((left, right, reverse, label), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, shaft, _): (usize, Range<usize>, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.separate((Axis::Stacked,shaft.len()), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, bars, _): (usize, Range<usize>, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.separate((Axis::SideBySide,bars.len()), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, Declaration, usize),
    (_, _opener, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, one, _): (usize, Name, usize),
) -> Result<Declaration,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.single(one).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, left, _): (usize, Name, usize),
    (_, _, _): (usize, Token, usize),
    (_, right, _): (usize, Name, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Declaration,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.alias((left, right), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Name, usize),
) -> Name
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, __0, _): (usize, (), usize),
) -> Name
{
    Name { text: "state".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Name, usize),
) -> Name
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Name
{
    Name { text: "state".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, value, _): (usize, Name, usize),
) -> Name
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Name
{
    Name { text: "as".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Name
{
    Name { text: "begin".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Name
{
    Name { text: "end".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action28<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Name
{
    Name { text: "endstate".into(), quoted: false }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action29<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
) -> Endpoint
{
    Endpoint::Named(path)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action30<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Endpoint
{
    Endpoint::Role
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action31<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, kind, _): (usize, Kind, usize),
) -> Endpoint
{
    Endpoint::History(kind)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action32<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
    (_, h, _): (usize, usize, usize),
    (_, kind, _): (usize, Kind, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    require_adjacent((path.end,h),h..r).map(|()|Endpoint::ScopedHistory(path,kind)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action33<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
) -> Endpoint
{
    Endpoint::Named(path)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action34<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Endpoint
{
    Endpoint::Role
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action35<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, kind, _): (usize, Kind, usize),
) -> Endpoint
{
    Endpoint::History(kind)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action36<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
    (_, h, _): (usize, usize, usize),
    (_, kind, _): (usize, Kind, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    require_adjacent((path.end,h),h..r).map(|()|Endpoint::ScopedHistory(path,kind)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action37<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Kind
{
    Kind::State
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action38<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, name, _): (usize, Name, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Kind,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.stereotype(name,start..end).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action39<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, at, _): (usize, usize, usize),
) -> Range<usize>
{
    at..at+1
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action40<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, shaft, _): (usize, Range<usize>, usize),
    (_, start, _): (usize, usize, usize),
    (_, at, _): (usize, usize, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    join_shaft(shaft,at,start..end).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action41<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Caption
{
    Caption::Absent
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action42<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, label, _): (usize, String, usize),
) -> Caption
{
    Caption::Present(label)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action43<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Caption
{
    Caption::Absent
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action44<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, _colon, _): (usize, (), usize),
    (_, label, _): (usize, String, usize),
) -> Caption
{
    Caption::Present(label)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action45<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action46<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action47<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, shaft, _): (usize, Range<usize>, usize),
    (_, start, _): (usize, usize, usize),
    (_, head, _): (usize, usize, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    require_adjacent((shaft.end, head), start..end).map(|()| false).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action48<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, head, _): (usize, usize, usize),
    (_, end, _): (usize, usize, usize),
    (_, shaft, _): (usize, Range<usize>, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    require_adjacent((head + 1, shaft.start), start..end).map(|()| true).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action49<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, at, _): (usize, usize, usize),
) -> Range<usize>
{
    at..at + 1
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action50<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, shaft, _): (usize, Range<usize>, usize),
    (_, start, _): (usize, usize, usize),
    (_, at, _): (usize, usize, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    join_shaft(shaft, at, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action51<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.path(name,start).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action52<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
    (_, dot, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.extend_path((path,name),(dot,start)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action53<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.path(name,start).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action54<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    (_, path, _): (usize, NamePath, usize),
    (_, dot, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.extend_path((path,name),(dot,start)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action55<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action56<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action57<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
    __2: (usize, usize, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action47(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action58<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, usize, usize),
    __2: (usize, Range<usize>, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action48(
        builder,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action59<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
    __2: (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action60<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Declaration, usize),
    __3: (usize, (), usize),
    __4: (usize, Token, usize),
    __5: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
        builder,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action61<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
    __2: (usize, Name, usize),
    __3: (usize, usize, usize),
) -> Result<Declaration,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action19(
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
fn __action62<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Kind, usize),
    __2: (usize, usize, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action63<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
    __2: (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action50(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action64<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Declaration, usize),
    __3: (usize, Kind, usize),
    __4: (usize, Caption, usize),
    __5: (usize, Token, usize),
    __6: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        builder,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action65<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, (), usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action6(
        builder,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action66<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Endpoint, usize),
    __1: (usize, bool, usize),
    __2: (usize, Endpoint, usize),
    __3: (usize, Caption, usize),
    __4: (usize, Token, usize),
    __5: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        builder,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action67<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, Token, usize),
    __2: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action8(
        builder,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action68<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, Token, usize),
    __2: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action9(
        builder,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action69<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Kind, usize),
    __2: (usize, usize, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action32(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action70<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action53(
        builder,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action71<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Token, usize),
    __2: (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __start1 = __1.2.clone();
    let __end1 = __2.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    let __temp1 = __action56(
        builder,
        &__start1,
        &__end1,
    );
    let __temp1 = (__start1, __temp1, __end1);
    __action54(
        builder,
        __0,
        __temp0,
        __1,
        __temp1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action72<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action51(
        builder,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action73<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Token, usize),
    __2: (usize, Name, usize),
) -> Result<NamePath,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __start1 = __1.2.clone();
    let __end1 = __2.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    let __temp1 = __action56(
        builder,
        &__start1,
        &__end1,
    );
    let __temp1 = (__start1, __temp1, __end1);
    __action52(
        builder,
        __0,
        __temp0,
        __1,
        __temp1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action74<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Name, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
) -> Result<Kind,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action56(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
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
fn __action75<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action57(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action76<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Range<usize>, usize),
) -> Result<bool,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action58(
        builder,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action77<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action59(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action78<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Declaration, usize),
    __3: (usize, (), usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action60(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action79<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
    __2: (usize, Name, usize),
) -> Result<Declaration,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action61(
        builder,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action80<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Kind, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action62(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action81<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, usize, usize),
) -> Result<Range<usize>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action63(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action82<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Declaration, usize),
    __3: (usize, Kind, usize),
    __4: (usize, Caption, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action64(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action83<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, (), usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action65(
        builder,
        __0,
        __1,
        __2,
        __3,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action84<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Endpoint, usize),
    __1: (usize, bool, usize),
    __2: (usize, Endpoint, usize),
    __3: (usize, Caption, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action66(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action85<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action67(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action86<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Range<usize>, usize),
    __1: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action68(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action87<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, NamePath, usize),
    __1: (usize, Kind, usize),
) -> Result<Endpoint,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action69(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action88<
>(
    builder: &mut crate::plantuml::state::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Name, usize),
    __2: (usize, Token, usize),
) -> Result<Kind,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action55(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action74(
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
