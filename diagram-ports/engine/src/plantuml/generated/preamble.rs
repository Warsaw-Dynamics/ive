// auto-generated: "lalrpop 0.23.2"
// sha3: 5fe941376c380611fc17b20bcbfad38a81211296feb1ddecb80e4cb7748483fc
// features: 
use crate::plantuml::{parser::{preamble::Token, error::ParseFailure}, style::{Alignment, Role, Target, PropertyKey}};
use std::ops::Range;
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Preamble {

    use crate::plantuml::{parser::{preamble::Token, error::ParseFailure}, style::{Alignment, Role, Target, PropertyKey}};
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
        Variant1((Target, PropertyKey)),
        Variant2(PropertyKey),
        Variant3(String),
        Variant4(Option<Token>),
        Variant5(usize),
        Variant6(Alignment),
        Variant7(Role),
        Variant8(()),
        Variant9(Range<usize>),
        Variant10((Role, Alignment)),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -26, -26, -26, -26, -26, -26, -26, -26, 0, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26,
        // State 1
        27, 24, 22, 21, 23, 20, 25, 5, 0, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19,
        // State 2
        0, 0, 30, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 35, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 5
        -16, -16, -16, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, 0, -16, 0, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14, 0, -14, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14, 0, -14, 0, 0, 0, 0, 0, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 44, 36, 35, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        57, 55, 54, 53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 58, 0, 0, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 60, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 61, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14, 0, -14, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 72, 0, 0, 0, 0, 0, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        -27, -27, -27, -27, -27, -27, -27, -27, 0, -27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 31, 0, 32, 0, 0, 0,
        // State 17
        -25, -25, -25, -25, -25, -25, -25, -25, 0, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32, 0, -32, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -31, 0, -31, 0, 0, 0,
        // State 22
        0, 0, -7, -7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30, 0, -30, 0, 0, 0,
        // State 24
        0, 0, -9, -9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29, 0, -29, 0, 0, 0,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -33, 0, -33, 0, 0, 0,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11, 0, -11, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -10, 0, -10, 0, 0, 0,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43, -43, -43, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46, 0, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -46, -46, -46, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -48, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 49, 0,
        // State 41
        -23, -23, -23, -23, -23, -23, -23, -23, 0, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44, -44, -44, -44, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 0,
        // State 44
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 0,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 0,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 0,
        // State 48
        -22, -22, -22, -22, -22, -22, -22, -22, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22,
        // State 49
        -18, -18, -18, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, -18, 0, 0, 0, 0, 0, 0,
        // State 50
        -17, -17, -17, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, 0, -17, 0, 0, 0, 0, 0, 0,
        // State 51
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 67, 0, 0, 0, 0, 0, 0, 0,
        // State 52
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -38, 0, 0, 0, 0, 0, 0, 0,
        // State 53
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -37, 0, 0, 0, 0, 0, 0, 0,
        // State 54
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -36, 0, 0, 0, 0, 0, 0, 0,
        // State 55
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 68, 0, 0, 0,
        // State 56
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -35, 0, 0, 0, 0, 0, 0, 0,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 69, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, 0, -15, 0, 0, 0, 0, 0, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 70, 0,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 0,
        // State 61
        -45, -45, -45, -45, -45, -45, -45, -45, 0, -45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -45,
        // State 62
        -20, -20, -20, -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, -20, 0, 0, 0, 0, 0, 0,
        // State 63
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, 0, -21, 0, 0, 0, 0, 0, 0,
        // State 64
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -39, 0, -39, 0, 0, 0, 0, 0, 0,
        // State 65
        -24, -24, -24, -24, -24, -24, -24, -24, 0, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24,
        // State 66
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 73, 0,
        // State 67
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 0, 0, 75, 0,
        // State 68
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -42, -42, -42, -42, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -41, -41, -41, -41, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 70
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -40, -40, -40, -40, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 71
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76, 0,
        // State 72
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34, 0, -34, 0, 0, 0, 0, 0, 0,
        // State 73
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 0,
        // State 74
        -13, -13, -13, -13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -13, 0, -13, 0, 0, 0, 0, 0, 0,
        // State 75
        -19, -19, -19, -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, -19, 0, 0, 0, 0, 0, 0,
        // State 76
        -12, -12, -12, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, 0, -12, 0, 0, 0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 23 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        0,
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
        -49,
        // State 15
        0,
        // State 16
        0,
        // State 17
        0,
        // State 18
        -28,
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
        0,
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
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            4 => 2,
            5 => 27,
            6 => match state {
                9 => 49,
                _ => 58,
            },
            7 => match state {
                7 => 11,
                12 => 13,
                _ => 10,
            },
            8 => 9,
            9 => 50,
            10 => 5,
            11 => 6,
            12 => 15,
            13 => 1,
            14 => 14,
            15 => 16,
            16 => 12,
            17 => 51,
            18 => 7,
            19 => match state {
                8 => 42,
                _ => 32,
            },
            20 => 8,
            21 => 17,
            22 => 3,
            23 => 37,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""title""###,
        r###""legend""###,
        r###""header""###,
        r###""footer""###,
        r###""left""###,
        r###""center""###,
        r###""right""###,
        r###""skinparam""###,
        r###""skin key""###,
        r###""sheet start""###,
        r###""sheet end""###,
        r###""root""###,
        r###""element""###,
        r###""document""###,
        r###""property""###,
        r###""{""###,
        r###""}""###,
        r###"":""###,
        r###"";""###,
        r###""value""###,
        r###""unknown""###,
        r###""newline""###,
        r###""body""###,
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
        builder: &'__1 mut crate::plantuml::parser::preamble::Builder<'__0>,
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
        type Success = Range<usize>;
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
            Token::Title if true => Some(0),
            Token::Legend if true => Some(1),
            Token::Header if true => Some(2),
            Token::Footer if true => Some(3),
            Token::Left if true => Some(4),
            Token::Center if true => Some(5),
            Token::Right if true => Some(6),
            Token::Skinparam if true => Some(7),
            Token::SkinKey(_) if true => Some(8),
            Token::SheetStart if true => Some(9),
            Token::SheetEnd if true => Some(10),
            Token::Root if true => Some(11),
            Token::Element if true => Some(12),
            Token::Document if true => Some(13),
            Token::Property(_) if true => Some(14),
            Token::Open if true => Some(15),
            Token::Close if true => Some(16),
            Token::Colon if true => Some(17),
            Token::Semi if true => Some(18),
            Token::Value(_) if true => Some(19),
            Token::Unknown if true => Some(20),
            Token::Newline if true => Some(21),
            Token::Body if true => Some(22),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10 | 11 | 12 | 13 | 15 | 16 | 17 | 18 | 20 | 21 | 22 => __Symbol::Variant0(__token),
            8 => match __token {
                Token::SkinKey(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            14 => match __token {
                Token::Property(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            19 => match __token {
                Token::Value(__tok0) if true => __Symbol::Variant3(__tok0),
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
                    states_to_pop: 1,
                    nonterminal_produced: 0,
                }
            }
            1 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 0,
                }
            }
            2 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 1,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 1,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 2,
                }
            }
            5 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
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
                    states_to_pop: 1,
                    nonterminal_produced: 4,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 5,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 5,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 6,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 6,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 7,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 7,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 8,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
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
                    states_to_pop: 4,
                    nonterminal_produced: 9,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 10,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 11,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 12,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 13,
                }
            }
            27 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 14,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            29 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 15,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 16,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 17,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 17,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 17,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 17,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 18,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 19,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 19,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 19,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 20,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 20,
                }
            }
            44 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 21,
                }
            }
            45 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 22,
                }
            }
            46 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 23,
                }
            }
            47 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 23,
                }
            }
            48 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct PreambleParser {
        _priv: (),
    }

    impl Default for PreambleParser { fn default() -> Self { Self::new() } }
    impl PreambleParser {
        pub fn new() -> PreambleParser {
            PreambleParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<Range<usize>, __lalrpop_util::ParseError<usize, Token, ParseFailure>>
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
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> Option<Result<Range<usize>,__lalrpop_util::ParseError<usize, Token, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            1 => {
                __reduce1(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            2 => {
                __reduce2(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            3 => {
                __reduce3(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            4 => {
                __reduce4(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                __reduce8(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            9 => {
                __reduce9(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            11 => {
                // Declaration = "property", "value", ";", "newline" => ActionFn(64);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action64::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 6)
            }
            12 => {
                // Declaration = "property", "value", "newline" => ActionFn(65);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action65::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (3, 6)
            }
            13 => {
                __reduce13(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            14 => {
                // Declarations = Declarations, Declaration => ActionFn(34);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant8(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action34::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (2, 7)
            }
            15 => {
                __reduce15(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            16 => {
                // DocumentDeclarations = DocumentDeclarations, DocumentItem => ActionFn(36);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant8(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action36::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (2, 8)
            }
            17 => {
                __reduce17(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            18 => {
                // DocumentItem = RegionOpen, Declarations, "}", "newline" => ActionFn(66);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action66::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 9)
            }
            19 => {
                // DocumentOpen = "document", "{", "newline" => ActionFn(32);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action32::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 10)
            }
            20 => {
                // ElementOpen = "element", "{", "newline" => ActionFn(31);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action31::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 11)
            }
            21 => {
                // Item = RegionHead, ":", "value", "newline" => ActionFn(67);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant10(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action67::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 12)
            }
            22 => {
                // Item = RegionHead, "value", "newline" => ActionFn(68);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant10(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action68::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (3, 12)
            }
            23 => {
                // Item = "skinparam", SkinKey, "value", "newline" => ActionFn(69);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action69::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 12)
            }
            24 => {
                __reduce24(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            25 => {
                __reduce25(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            26 => {
                // Items = Items, Item => ActionFn(3);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant8(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action3::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (2, 13)
            }
            27 => {
                __reduce27(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            28 => {
                __reduce28(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            29 => {
                __reduce29(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            30 => {
                __reduce30(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            31 => {
                __reduce31(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            32 => {
                __reduce32(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            33 => {
                // RegionOpen = RegionRole, "{", "newline" => ActionFn(39);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant7(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action39::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 16)
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
                __reduce37(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            38 => {
                // RootOpen = "root", "{", "newline" => ActionFn(30);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action30::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 18)
            }
            39 => {
                // Rule = RootOpen, Declarations, "}", "newline" => ActionFn(71);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action71::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 19)
            }
            40 => {
                // Rule = ElementOpen, Declarations, "}", "newline" => ActionFn(72);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action72::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 19)
            }
            41 => {
                // Rule = DocumentOpen, DocumentDeclarations, "}", "newline" => ActionFn(73);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action73::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 19)
            }
            42 => {
                __reduce42(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            43 => {
                // Rules = Rules, Rule => ActionFn(26);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant8(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action26::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (2, 20)
            }
            44 => {
                // Sheet = SheetOpen, Rules, "sheet end", "newline" => ActionFn(74);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant8(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action74::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 21)
            }
            45 => {
                // SheetOpen = "sheet start", "newline" => ActionFn(75);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action75::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (2, 22)
            }
            46 => {
                __reduce46(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            47 => {
                // SkinKey = "property" => ActionFn(76);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action76::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant1(__nt), __end));
                (1, 23)
            }
            48 => {
                // __Preamble = Preamble => ActionFn(0);
                let __sym0 = __pop_Variant9(__symbols);
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
    fn __pop_Variant8<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (Role, Alignment), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (Target, PropertyKey), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Alignment, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Option<Token>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, PropertyKey, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Range<usize>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Role, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
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
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ":"? = ":" => ActionFn(43);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action43::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 0)
    }
    fn __reduce1<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ":"? =  => ActionFn(44);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action44::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 0)
    }
    fn __reduce2<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ";"? = ";" => ActionFn(41);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action41::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 1)
    }
    fn __reduce3<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // ";"? =  => ActionFn(42);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action42::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 1)
    }
    fn __reduce4<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(46);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action46::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 2)
    }
    fn __reduce5<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(45);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action45::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 3)
    }
    fn __reduce6<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Align = "left" => ActionFn(12);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 4)
    }
    fn __reduce7<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Align = "center" => ActionFn(13);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action13::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 4)
    }
    fn __reduce8<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Align = "right" => ActionFn(14);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action14::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 4)
    }
    fn __reduce9<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // AlignedRole = "header" => ActionFn(15);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action15::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 5)
    }
    fn __reduce10<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // AlignedRole = "footer" => ActionFn(16);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action16::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 5)
    }
    fn __reduce13<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Declarations =  => ActionFn(33);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action33::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 7)
    }
    fn __reduce15<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DocumentDeclarations =  => ActionFn(35);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action35::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 8)
    }
    fn __reduce17<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // DocumentItem = Declaration => ActionFn(37);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action37::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 9)
    }
    fn __reduce24<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Item = Sheet => ActionFn(6);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action6::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 12)
    }
    fn __reduce25<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Items =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 13)
    }
    fn __reduce27<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Preamble = Items, "body" => ActionFn(70);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action70::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (2, 14)
    }
    fn __reduce28<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionHead = "title" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce29<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionHead = "legend" => ActionFn(8);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce30<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionHead = "header" => ActionFn(9);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce31<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionHead = "footer" => ActionFn(10);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    fn __reduce32<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionHead = Align, AlignedRole => ActionFn(11);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action11::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (2, 15)
    }
    fn __reduce34<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionRole = "title" => ActionFn(17);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 17)
    }
    fn __reduce35<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionRole = "legend" => ActionFn(18);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 17)
    }
    fn __reduce36<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionRole = "header" => ActionFn(19);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action19::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 17)
    }
    fn __reduce37<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // RegionRole = "footer" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 17)
    }
    fn __reduce42<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Rules = Rule => ActionFn(25);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action25::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 20)
    }
    fn __reduce46<
    >(
        builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // SkinKey = "skin key" => ActionFn(21);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action21::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 23)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Preamble::PreambleParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Range<usize>, usize),
) -> Range<usize>
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, _items, _): (usize, (), usize),
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Range<usize>
{
    start..end
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
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
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, head, _): (usize, (Role, Alignment), usize),
    (_, _, _): (usize, Option<Token>, usize),
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.region(head, value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, key, _): (usize, (Target, PropertyKey), usize),
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.skin(key, value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> (Role, Alignment)
{
    (Role::Title, Alignment::Center)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> (Role, Alignment)
{
    (Role::Legend, Alignment::Center)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> (Role, Alignment)
{
    (Role::Header, Alignment::Right)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> (Role, Alignment)
{
    (Role::Footer, Alignment::Center)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, align, _): (usize, Alignment, usize),
    (_, role, _): (usize, Role, usize),
) -> (Role, Alignment)
{
    (role, align)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Alignment
{
    Alignment::Left
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Alignment
{
    Alignment::Center
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Alignment
{
    Alignment::Right
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Header
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Footer
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Title
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Legend
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Header
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Role
{
    Role::Footer
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, key, _): (usize, (Target, PropertyKey), usize),
) -> (Target, PropertyKey)
{
    key
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, key, _): (usize, PropertyKey, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(Target, PropertyKey),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    if key == PropertyKey::BackgroundColor { Ok((Target::Document, key)) }
        else { Err(lalrpop_util::ParseError::User { error: ParseFailure::unsupported(start..end, "unsupported skinparam property") }) }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _open, _): (usize, (), usize),
    (_, _rules, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.finish_sheet(start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.sheet(start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.tick().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, count, _): (usize, usize, usize),
    (_, _declarations, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close(count, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action28<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, count, _): (usize, usize, usize),
    (_, _declarations, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close(count, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action29<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, count, _): (usize, usize, usize),
    (_, _declarations, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close(count, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action30<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
    (_, __2, _): (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(Target::Root, 1).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action31<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
    (_, __2, _): (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(Target::Element, 1).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action32<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
    (_, __2, _): (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(Target::Document, 1).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action33<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action34<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.tick().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action35<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action36<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.tick().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action37<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action38<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, count, _): (usize, usize, usize),
    (_, _declarations, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close(count, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action39<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, role, _): (usize, Role, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(Target::Region(role), 2).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action40<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, key, _): (usize, PropertyKey, usize),
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Option<Token>, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.declare(key, value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action41<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Option<Token>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action42<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Token>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action43<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Option<Token>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action44<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Token>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action45<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action46<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action47<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, Token, usize),
    __2: (usize, usize, usize),
    __3: (usize, String, usize),
    __4: (usize, usize, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action43(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action48<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, usize, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action44(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        builder,
        __0,
        __temp0,
        __1,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action49<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, usize, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __4.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action41(
        builder,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __1,
        __2,
        __3,
        __temp0,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action50<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, usize, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __4.0.clone();
    let __temp0 = __action42(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __1,
        __2,
        __3,
        __temp0,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action51<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, String, usize),
    __2: (usize, usize, usize),
    __3: (usize, Token, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action49(
        builder,
        __0,
        __temp0,
        __1,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action52<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, String, usize),
    __2: (usize, usize, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action46(
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
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action53<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
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
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action54<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action47(
        builder,
        __0,
        __1,
        __temp0,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action55<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, String, usize),
    __2: (usize, usize, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action48(
        builder,
        __0,
        __temp0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action56<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, (Target, PropertyKey), usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action5(
        builder,
        __0,
        __1,
        __temp0,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action57<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, Token, usize),
    __2: (usize, usize, usize),
) -> Range<usize>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
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
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
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
fn __action59<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
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
fn __action60<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action29(
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
fn __action61<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action23(
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
fn __action62<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action24(
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
fn __action63<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, usize, usize),
) -> Result<(Target, PropertyKey),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action46(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action64<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action51(
        builder,
        __0,
        __1,
        __temp0,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action65<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action52(
        builder,
        __0,
        __1,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action66<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action53(
        builder,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action67<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action54(
        builder,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action68<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (Role, Alignment), usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action55(
        builder,
        __0,
        __1,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action69<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, (Target, PropertyKey), usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action56(
        builder,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action70<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, Token, usize),
) -> Range<usize>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
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
fn __action71<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action58(
        builder,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action72<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action59(
        builder,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action73<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
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
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action74<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action45(
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
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action75<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action62(
        builder,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action76<
>(
    builder: &mut crate::plantuml::parser::preamble::Builder<'_>,
    __0: (usize, PropertyKey, usize),
) -> Result<(Target, PropertyKey),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action45(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action63(
        builder,
        __0,
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
