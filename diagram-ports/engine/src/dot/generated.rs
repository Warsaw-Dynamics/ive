// auto-generated: "lalrpop 0.23.2"
// sha3: b47f9bef0c3cf2ed555e35aecb9a3de42c790aaca4c173c8de6dae3e9b0d79b0
// features: 
use std::cell::Cell;
use crate::dot::{DotError, ParseOptions, Token, ast::{statement_from_endpoint, concat_quoted, Attribute, AttributeScope, Document, EdgeOp, Endpoint, GraphKind, Id, NodeId, Statement, Subgraph}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Document {

    use std::cell::Cell;
    use crate::dot::{DotError, ParseOptions, Token, ast::{statement_from_endpoint, concat_quoted, Attribute, AttributeScope, Document, EdgeOp, Endpoint, GraphKind, Id, NodeId, Statement, Subgraph}};
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
        Variant1(Id),
        Variant2(Option<Token>),
        Variant3(NodeId),
        Variant4(alloc::vec::Vec<NodeId>),
        Variant5(Option<Id>),
        Variant6(usize),
        Variant7(Attribute),
        Variant8(alloc::vec::Vec<Attribute>),
        Variant9(Vec<Attribute>),
        Variant10(alloc::vec::Vec<Vec<Attribute>>),
        Variant11(()),
        Variant12(Document),
        Variant13((EdgeOp, Endpoint)),
        Variant14(alloc::vec::Vec<(EdgeOp, Endpoint)>),
        Variant15(Endpoint),
        Variant16(GraphKind),
        Variant17(Vec<Id>),
        Variant18(Option<Vec<Id>>),
        Variant19(alloc::vec::Vec<Id>),
        Variant20(Statement),
        Variant21(alloc::vec::Vec<Statement>),
        Variant22(bool),
        Variant23(Subgraph),
    }
    const __ACTION: &[i8] = &[
        // State 0
        47, -80, -80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        0, 49, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 4, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, -42, 0, -42, -42, -42, -42, -42, -42, -42, -42, -42, -42, -42, -42, 52, -42, -42,
        // State 4
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 57, 0, 0, 0, 0, 0, 0, 0,
        // State 5
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 58, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        0, -43, 0, -43, -43, -43, -43, -43, -43, -43, -43, -43, -43, -43, -43, 52, -43, -43,
        // State 7
        0, -25, 0, -25, -25, -25, -25, 21, 20, -25, -25, 22, 0, 0, 0, 0, 0, -25,
        // State 8
        0, -52, 0, -52, -52, -52, -52, -52, -52, -52, -52, -52, 0, 24, 23, 0, -52, -52,
        // State 9
        0, -53, 0, -53, -53, -53, -53, -53, -53, -53, -53, -53, 0, 0, 0, 0, 25, -53,
        // State 10
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 68, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 4, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 71, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 72, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, -26, 0, -26, -26, -26, -26, 0, 0, -26, -26, 22, 0, 0, 0, 0, 0, -26,
        // State 18
        0, -25, 0, -25, -25, -25, -25, 21, 20, -25, -25, 22, 0, 0, 0, 0, 0, -25,
        // State 19
        0, 0, 0, 0, 0, 15, 4, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 20
        0, 0, 0, 0, 0, 15, 4, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, -18, 0, 0, 0, 0, 0,
        // State 22
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 25
        0, -69, 0, -69, -69, -69, -69, 0, 0, -69, -69, 22, 0, 0, 0, 0, 0, -69,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 27
        0, -65, 0, -65, -65, -65, -65, 0, 0, -65, -65, 22, 0, 0, 0, 0, 0, -65,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 29
        0, -67, 0, -67, -67, -67, -67, 0, 0, -67, -67, 22, 0, 0, 0, 0, 0, -67,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 31
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 85, 0, 0, 0, 0, 0, 0, 0,
        // State 32
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 86, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        0, -52, 0, -52, -52, -52, -52, -52, -52, -52, -52, -52, 0, 0, 23, 0, -52, -52,
        // State 34
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, -19, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        0, -68, 0, -68, -68, -68, -68, 0, 0, -68, -68, 22, 0, 0, 0, 0, 0, -68,
        // State 37
        0, -64, 0, -64, -64, -64, -64, 0, 0, -64, -64, 22, 0, 0, 0, 0, 0, -64,
        // State 38
        0, -66, 0, -66, -66, -66, -66, 0, 0, -66, -66, 22, 0, 0, 0, 0, 0, -66,
        // State 39
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 90, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 91, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 13, 0, 14, 12, 15, 4, 0, 0, 16, 93, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        0, 0, 0, 0, 0, 0, -29, 0, 0, 0, 0, 0, -29, 0, 0, 0, 95, 96,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 46
        0, -79, -79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 47
        0, 0, 0, 0, 0, 0, -47, 0, 0, -47, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 48
        0, 0, 0, 0, 0, 0, -46, 0, 0, -46, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 50
        0, -62, 0, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62, -62,
        // State 51
        0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 52
        0, -40, 0, -40, -40, -40, -40, -40, -40, -40, -40, -40, 0, 0, 0, 0, 0, -40,
        // State 53
        0, -74, 0, -74, -74, -74, -74, 0, 0, -74, -74, 0, 0, 0, 0, 0, 0, 66,
        // State 54
        0, -77, 0, -77, -77, -77, -77, 0, 0, -77, -77, 0, 0, 0, 0, 0, 0, 0,
        // State 55
        0, -41, 0, -41, -41, -41, -41, -41, -41, -41, -41, -41, 0, 0, 0, 0, 0, -41,
        // State 56
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 58
        0, -63, 0, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63, -63,
        // State 59
        0, -59, 0, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59, -59,
        // State 60
        0, -23, 0, -23, -23, -23, -23, 0, 0, -23, -23, -23, 0, 0, 0, 0, 0, -23,
        // State 61
        0, -71, 0, -71, -71, -71, -71, 0, 0, -71, -71, 0, 0, 0, 0, 0, 0, -71,
        // State 62
        0, -38, 0, -38, -38, -38, -38, -38, -38, -38, -38, -38, 0, 0, 0, 0, 0, -38,
        // State 63
        0, -51, 0, -51, -51, -51, -51, -51, -51, -51, -51, -51, 0, 0, 0, 0, -51, -51,
        // State 64
        0, -54, 0, -54, -54, -54, -54, -54, -54, -54, -54, -54, 0, 0, 0, 0, 36, -54,
        // State 65
        0, -73, 0, -73, -73, -73, -73, 0, 0, -73, -73, 0, 0, 0, 0, 0, 0, 0,
        // State 66
        0, -78, 0, -78, -78, -78, -78, 0, 0, -78, -78, 0, 0, 0, 0, 0, 0, 0,
        // State 67
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 68
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 0, 0, 0, 0,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 70
        0, -85, 0, -85, -85, -85, -85, -85, -85, -85, -85, -85, 0, 0, 0, 0, 0, -85,
        // State 71
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 72
        0, -24, 0, -24, -24, -24, -24, 0, 0, -24, -24, -24, 0, 0, 0, 0, 0, -24,
        // State 73
        0, -72, 0, -72, -72, -72, -72, 0, 0, -72, -72, 0, 0, 0, 0, 0, 0, -72,
        // State 74
        0, -39, 0, -39, -39, -39, -39, -39, -39, -39, -39, -39, 0, 0, 0, 0, 0, -39,
        // State 75
        0, -35, 0, -35, -35, -35, -35, -35, -35, -35, -35, -35, 0, 0, 0, 0, 0, -35,
        // State 76
        0, -34, 0, -34, -34, -34, -34, -34, -34, -34, -34, -34, 0, 0, 0, 0, 0, -34,
        // State 77
        0, 0, 0, 0, 0, 0, -16, 0, 0, 0, 0, 0, -16, 0, 0, 0, 0, 0,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0, 0, 0, 0,
        // State 79
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0,
        // State 80
        0, -56, 0, -56, -56, -56, -56, -56, -56, -56, -56, -56, 0, 0, 43, 0, -56, -56,
        // State 81
        0, -70, 0, -70, -70, -70, -70, 0, 0, -70, -70, 0, 0, 0, 0, 0, 0, -70,
        // State 82
        0, -6, 0, -6, -6, -6, -6, -6, -6, -6, -6, -6, 0, 0, 0, 0, -6, -6,
        // State 83
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -48, 0, 0, 0, 0, 0, 0,
        // State 84
        0, -83, 0, -83, -83, -83, -83, -83, -83, -83, -83, -83, 0, 0, 0, 0, 0, -83,
        // State 85
        0, -86, 0, -86, -86, -86, -86, -86, -86, -86, -86, -86, 0, 0, 0, 0, 0, -86,
        // State 86
        0, 0, 0, 0, 0, 0, -17, 0, 0, 0, 0, 0, -17, 0, 0, 0, 0, 0,
        // State 87
        0, -20, 0, -20, -20, -20, -20, 0, 0, -20, -20, -20, 0, 0, 0, 0, 0, -20,
        // State 88
        0, -7, 0, -7, -7, -7, -7, -7, -7, -7, -7, -7, 0, 0, 0, 0, -7, -7,
        // State 89
        0, -81, 0, -81, -81, -81, -81, -81, -81, -81, -81, -81, 0, 0, 0, 0, 0, -81,
        // State 90
        0, -84, 0, -84, -84, -84, -84, -84, -84, -84, -84, -84, 0, 0, 0, 0, 0, -84,
        // State 91
        0, -55, 0, -55, -55, -55, -55, -55, -55, -55, -55, -55, 0, 0, 0, 0, -55, -55,
        // State 92
        0, -82, 0, -82, -82, -82, -82, -82, -82, -82, -82, -82, 0, 0, 0, 0, 0, -82,
        // State 93
        0, 0, 0, 0, 0, 0, -13, 0, 0, 0, 0, 0, -13, 0, 0, 0, 0, 0,
        // State 94
        0, 0, 0, 0, 0, 0, -28, 0, 0, 0, 0, 0, -28, 0, 0, 0, 0, 0,
        // State 95
        0, 0, 0, 0, 0, 0, -27, 0, 0, 0, 0, 0, -27, 0, 0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 18 + integer]
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
        -87,
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
        -32,
        // State 57
        -30,
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
        -33,
        // State 68
        0,
        // State 69
        0,
        // State 70
        0,
        // State 71
        -31,
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
        0,
        // State 80
        0,
        // State 81
        0,
        // State 82
        0,
        // State 83
        0,
        // State 84
        0,
        // State 85
        0,
        // State 86
        0,
        // State 87
        0,
        // State 88
        0,
        // State 89
        0,
        // State 90
        0,
        // State 91
        0,
        // State 92
        0,
        // State 93
        0,
        // State 94
        0,
        // State 95
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            3 => 64,
            8 => match state {
                34 => 86,
                _ => 77,
            },
            10 => 34,
            11 => 78,
            12 => match state {
                17 | 25 | 27 | 29 | 36..=38 => 72,
                _ => 60,
            },
            14 => match state {
                11 => 25,
                12 => 27,
                13 => 29,
                26 => 36,
                28 => 37,
                30 => 38,
                _ => 17,
            },
            15 => match state {
                18 => 73,
                _ => 61,
            },
            16 => 93,
            17 => 45,
            18 => match state {
                18 => 74,
                _ => 62,
            },
            20 => 18,
            21 => match state {
                19 => 75,
                20 => 76,
                _ => 7,
            },
            22 => match state {
                19..=20 | 24 | 35 => 33,
                41 => 44,
                2 => 49,
                11..=13 => 68,
                14 => 69,
                21 | 34 => 79,
                22 => 80,
                23 => 81,
                42 => 91,
                _ => 8,
            },
            24 => 2,
            25 => match state {
                12 => 28,
                13 => 30,
                _ => 26,
            },
            27 => match state {
                24 => 82,
                35 => 88,
                _ => 9,
            },
            28 => 52,
            29 => 63,
            31 => match state {
                6 => 58,
                _ => 50,
            },
            33 => 6,
            34 => 53,
            35 => match state {
                4..=5 | 15 | 31 | 39 => 54,
                _ => 66,
            },
            37 => match state {
                5 => 16,
                15 => 32,
                31 => 40,
                39 => 43,
                _ => 10,
            },
            38 => 1,
            39 => 55,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""strict""###,
        r###""graph""###,
        r###""digraph""###,
        r###""node""###,
        r###""edge""###,
        r###""subgraph""###,
        r###""id""###,
        r###""->""###,
        r###""--""###,
        r###""{""###,
        r###""}""###,
        r###""[""###,
        r###""]""###,
        r###""=""###,
        r###"":""###,
        r###""+""###,
        r###"",""###,
        r###"";""###,
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
        'input,
    >(
        __states: &[i8],
        _: core::marker::PhantomData<(&'input ())>,
    ) -> alloc::vec::Vec<alloc::string::String>
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<(&())>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'input>
    where 
    {
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __phantom: core::marker::PhantomData<(&'input ())>,
    }
    impl<'input> __state_machine::ParserDefinition for __StateMachine<'input>
    where 
    {
        type Location = usize;
        type Error = DotError;
        type Token = Token;
        type TokenIndex = usize;
        type Symbol = __Symbol<>;
        type Success = Document;
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
            __token_to_integer(token, core::marker::PhantomData::<(&())>)
        }

        #[inline]
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 18 - 1)
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
            __token_to_symbol(token_index, token, core::marker::PhantomData::<(&())>)
        }

        fn expected_tokens(&self, state: i8) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens(state)
        }

        fn expected_tokens_from_states(&self, states: &[i8]) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens_from_states(states, core::marker::PhantomData::<(&())>)
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
                self.options,
                self.statement_count,
                action,
                start_location,
                states,
                symbols,
                core::marker::PhantomData::<(&())>,
            )
        }

        fn simulate_reduce(&self, action: i8) -> __state_machine::SimulatedReduce<Self> {
            __simulate_reduce(action, core::marker::PhantomData::<(&())>)
        }
    }
    fn __token_to_integer<
        'input,
    >(
        __token: &Token,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<usize>
    {
        #[warn(unused_variables)]
        match __token {
            Token::Strict if true => Some(0),
            Token::Graph if true => Some(1),
            Token::Digraph if true => Some(2),
            Token::Node if true => Some(3),
            Token::Edge if true => Some(4),
            Token::Subgraph if true => Some(5),
            Token::Id(_) if true => Some(6),
            Token::Arrow if true => Some(7),
            Token::Dash if true => Some(8),
            Token::Open if true => Some(9),
            Token::Close if true => Some(10),
            Token::OpenAttrs if true => Some(11),
            Token::CloseAttrs if true => Some(12),
            Token::Equal if true => Some(13),
            Token::Colon if true => Some(14),
            Token::Plus if true => Some(15),
            Token::Comma if true => Some(16),
            Token::Semicolon if true => Some(17),
            _ => None,
        }
    }
    fn __token_to_symbol<
        'input,
    >(
        __token_index: usize,
        __token: Token,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> __Symbol<>
    {
        #[allow(clippy::manual_range_patterns)]match __token_index {
            0 | 1 | 2 | 3 | 4 | 5 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 => __Symbol::Variant0(__token),
            6 => match __token {
                Token::Id(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        'input,
    >(
        __reduce_index: i8,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'input>>
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
                    states_to_pop: 2,
                    nonterminal_produced: 1,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 2,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 2,
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
                    states_to_pop: 3,
                    nonterminal_produced: 3,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 4,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 5,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
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
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 9,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 10,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 11,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 14,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 14,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 15,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 16,
                }
            }
            27 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 16,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 16,
                }
            }
            29 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 17,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 17,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 17,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 17,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 18,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 18,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 19,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 19,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 20,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 20,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 21,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 22,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 22,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 23,
                }
            }
            44 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 23,
                }
            }
            45 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 24,
                }
            }
            46 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 24,
                }
            }
            47 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 25,
                }
            }
            48 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 26,
                }
            }
            49 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 26,
                }
            }
            50 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 27,
                }
            }
            51 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 27,
                }
            }
            52 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 28,
                }
            }
            53 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 28,
                }
            }
            54 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 29,
                }
            }
            55 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 29,
                }
            }
            56 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 30,
                }
            }
            57 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 30,
                }
            }
            58 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 31,
                }
            }
            59 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 32,
                }
            }
            60 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 32,
                }
            }
            61 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 33,
                }
            }
            62 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 33,
                }
            }
            63 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 34,
                }
            }
            64 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 34,
                }
            }
            65 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 34,
                }
            }
            66 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 34,
                }
            }
            67 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 34,
                }
            }
            68 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 34,
                }
            }
            69 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 34,
                }
            }
            70 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 34,
                }
            }
            71 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 34,
                }
            }
            72 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 35,
                }
            }
            73 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 35,
                }
            }
            74 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 36,
                }
            }
            75 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 36,
                }
            }
            76 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 37,
                }
            }
            77 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 37,
                }
            }
            78 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 38,
                }
            }
            79 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 38,
                }
            }
            80 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 39,
                }
            }
            81 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 39,
                }
            }
            82 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 39,
                }
            }
            83 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 39,
                }
            }
            84 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 39,
                }
            }
            85 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 39,
                }
            }
            86 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct DocumentParser {
        _priv: (),
    }

    impl Default for DocumentParser { fn default() -> Self { Self::new() } }
    impl DocumentParser {
        pub fn new() -> DocumentParser {
            DocumentParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            'input,
            __TOKEN: __ToTriple<'input, >,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            options: &'input ParseOptions,
            statement_count: &'input Cell<usize>,
            __tokens0: __TOKENS,
        ) -> Result<Document, __lalrpop_util::ParseError<usize, Token, DotError>>
        {
            let __tokens = __tokens0.into_iter();
            let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
            __state_machine::Parser::drive(
                __StateMachine {
                    options,
                    statement_count,
                    __phantom: core::marker::PhantomData::<(&())>,
                },
                __tokens,
            )
        }
    }
    fn __accepts<
        'input,
    >(
        __error_state: Option<i8>,
        __states: &[i8],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> bool
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
            let (__to_pop, __nt) = match __simulate_reduce(-(__action + 1), core::marker::PhantomData::<(&())>) {
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
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<Result<Document,__lalrpop_util::ParseError<usize, Token, DotError>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                __reduce2(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            3 => {
                __reduce3(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            4 => {
                __reduce4(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            5 => {
                __reduce5(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            6 => {
                __reduce6(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            7 => {
                __reduce7(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            8 => {
                __reduce8(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            9 => {
                __reduce9(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            10 => {
                __reduce10(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            11 => {
                __reduce11(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            12 => {
                __reduce12(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            13 => {
                __reduce13(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            14 => {
                __reduce14(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            15 => {
                __reduce15(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            16 => {
                __reduce16(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            17 => {
                __reduce17(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            18 => {
                __reduce18(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            19 => {
                __reduce19(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            20 => {
                __reduce20(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            21 => {
                __reduce21(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            22 => {
                __reduce22(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            23 => {
                __reduce23(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            24 => {
                __reduce24(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            25 => {
                __reduce25(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            26 => {
                __reduce26(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            27 => {
                __reduce27(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            28 => {
                __reduce28(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            29 => {
                __reduce29(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            30 => {
                __reduce30(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            31 => {
                __reduce31(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            32 => {
                __reduce32(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            33 => {
                __reduce33(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            34 => {
                __reduce34(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            35 => {
                __reduce35(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            36 => {
                __reduce36(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            37 => {
                __reduce37(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            38 => {
                __reduce38(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            39 => {
                __reduce39(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            40 => {
                __reduce40(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            41 => {
                // Id = "id" => ActionFn(104);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action104::<>(options, statement_count, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant1(__nt), __end));
                (1, 22)
            }
            42 => {
                // Id = "id", QuotedTail+ => ActionFn(105);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant19(__symbols);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action105::<>(options, statement_count, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant1(__nt), __end));
                (2, 22)
            }
            43 => {
                __reduce43(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            44 => {
                __reduce44(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            45 => {
                __reduce45(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            46 => {
                __reduce46(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            47 => {
                __reduce47(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            48 => {
                __reduce48(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            49 => {
                __reduce49(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            50 => {
                __reduce50(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            51 => {
                __reduce51(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            52 => {
                __reduce52(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            53 => {
                __reduce53(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            54 => {
                __reduce54(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            55 => {
                __reduce55(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            56 => {
                __reduce56(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            57 => {
                __reduce57(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            58 => {
                __reduce58(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            59 => {
                __reduce59(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            60 => {
                __reduce60(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            61 => {
                __reduce61(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            62 => {
                __reduce62(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            63 => {
                __reduce63(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            64 => {
                __reduce64(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            65 => {
                __reduce65(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            66 => {
                __reduce66(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            67 => {
                __reduce67(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            68 => {
                __reduce68(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            69 => {
                __reduce69(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            70 => {
                // Statement = Endpoint, AttributeLists => ActionFn(90);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant9(__symbols);
                let __sym0 = __pop_Variant15(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action90::<>(options, statement_count, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant20(__nt), __end));
                (2, 34)
            }
            71 => {
                // Statement = Endpoint, EdgeStep+, AttributeLists => ActionFn(91);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant9(__symbols);
                let __sym1 = __pop_Variant14(__symbols);
                let __sym0 = __pop_Variant15(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action91::<>(options, statement_count, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant20(__nt), __end));
                (3, 34)
            }
            72 => {
                // StatementItem = Statement, ";" => ActionFn(84);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant20(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action84::<>(options, statement_count, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant20(__nt), __end));
                (2, 35)
            }
            73 => {
                // StatementItem = Statement => ActionFn(85);
                let __sym0 = __pop_Variant20(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action85::<>(options, statement_count, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant20(__nt), __end));
                (1, 35)
            }
            74 => {
                __reduce74(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            75 => {
                __reduce75(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            76 => {
                __reduce76(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            77 => {
                __reduce77(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            78 => {
                __reduce78(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            79 => {
                __reduce79(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            80 => {
                __reduce80(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            81 => {
                __reduce81(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            82 => {
                __reduce82(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            83 => {
                __reduce83(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            84 => {
                __reduce84(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            85 => {
                __reduce85(options, statement_count, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            86 => {
                // __Document = Document => ActionFn(0);
                let __sym0 = __pop_Variant12(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(options, statement_count, __sym0);
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
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant13<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (EdgeOp, Endpoint), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant13(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Attribute, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant12<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Document, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant15<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Endpoint, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant15(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, GraphKind, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant16(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Id, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, NodeId, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Option<Id>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Option<Token>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant18<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Option<Vec<Id>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant18(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant20<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Statement, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant20(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant23<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Subgraph, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant23(__v), __r)) => (__l, __v, __r),
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
    fn __pop_Variant9<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<Attribute>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant17<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<Id>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant17(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant14<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant14(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<Attribute>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant19<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<Id>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant19(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<NodeId>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant21<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<Statement>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant21(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, alloc::vec::Vec<Vec<Attribute>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant22<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, bool, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant22(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ";"? = ";" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 0)
    }
    fn __reduce1<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ";"? =  => ActionFn(46);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action46::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 0)
    }
    fn __reduce2<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ("," <NodeId>) = ",", NodeId => ActionFn(44);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action44::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 1)
    }
    fn __reduce3<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ("," <NodeId>)* =  => ActionFn(42);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action42::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 2)
    }
    fn __reduce4<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ("," <NodeId>)* = ("," <NodeId>)+ => ActionFn(43);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action43::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 2)
    }
    fn __reduce5<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ("," <NodeId>)+ = ",", NodeId => ActionFn(71);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action71::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 3)
    }
    fn __reduce6<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ("," <NodeId>)+ = ("," <NodeId>)+, ",", NodeId => ActionFn(72);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant3(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action72::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (3, 3)
    }
    fn __reduce7<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (":" <Id>) = ":", Id => ActionFn(39);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action39::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 4)
    }
    fn __reduce8<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (":" <Id>)? = ":", Id => ActionFn(75);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action75::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 5)
    }
    fn __reduce9<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (":" <Id>)? =  => ActionFn(38);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action38::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
    }
    fn __reduce10<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(50);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action50::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 6)
    }
    fn __reduce11<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(47);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action47::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
    }
    fn __reduce12<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Attribute = Id, "=", Id, AttributeSeparator => ActionFn(25);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant11(__symbols);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action25::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (4, 8)
    }
    fn __reduce13<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Attribute* =  => ActionFn(33);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action33::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 9)
    }
    fn __reduce14<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Attribute* = Attribute+ => ActionFn(34);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action34::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 9)
    }
    fn __reduce15<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Attribute+ = Attribute => ActionFn(65);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action65::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 10)
    }
    fn __reduce16<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Attribute+ = Attribute+, Attribute => ActionFn(66);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action66::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 10)
    }
    fn __reduce17<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeEntries =  => ActionFn(86);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action86::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 11)
    }
    fn __reduce18<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeEntries = Attribute+ => ActionFn(87);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action87::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 11)
    }
    fn __reduce19<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeList = "[", AttributeEntries, "]" => ActionFn(23);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant9(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action23::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (3, 12)
    }
    fn __reduce20<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeList* =  => ActionFn(35);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action35::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (0, 13)
    }
    fn __reduce21<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeList* = AttributeList+ => ActionFn(36);
        let __sym0 = __pop_Variant10(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action36::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 13)
    }
    fn __reduce22<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeList+ = AttributeList => ActionFn(51);
        let __sym0 = __pop_Variant9(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action51::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 14)
    }
    fn __reduce23<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeList+ = AttributeList+, AttributeList => ActionFn(52);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant9(__symbols);
        let __sym0 = __pop_Variant10(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action52::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (2, 14)
    }
    fn __reduce24<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeLists =  => ActionFn(88);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action88::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 15)
    }
    fn __reduce25<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeLists = AttributeList+ => ActionFn(89);
        let __sym0 = __pop_Variant10(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action89::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 15)
    }
    fn __reduce26<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeSeparator = ";" => ActionFn(26);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 16)
    }
    fn __reduce27<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeSeparator = "," => ActionFn(27);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action27::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 16)
    }
    fn __reduce28<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // AttributeSeparator =  => ActionFn(28);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action28::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 16)
    }
    fn __reduce29<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Document = Strict, Kind, Id, "{", "}" => ActionFn(106);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant16(__symbols);
        let __sym0 = __pop_Variant22(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action106::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (5, 17)
    }
    fn __reduce30<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Document = Strict, Kind, Id, "{", StatementItem+, "}" => ActionFn(107);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant21(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant16(__symbols);
        let __sym0 = __pop_Variant22(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action107::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (6, 17)
    }
    fn __reduce31<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Document = Strict, Kind, "{", "}" => ActionFn(108);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant16(__symbols);
        let __sym0 = __pop_Variant22(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action108::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (4, 17)
    }
    fn __reduce32<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Document = Strict, Kind, "{", StatementItem+, "}" => ActionFn(109);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant21(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant16(__symbols);
        let __sym0 = __pop_Variant22(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action109::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (5, 17)
    }
    fn __reduce33<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep = "->", Endpoint => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant15(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (2, 18)
    }
    fn __reduce34<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep = "--", Endpoint => ActionFn(14);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant15(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action14::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (2, 18)
    }
    fn __reduce35<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep* =  => ActionFn(48);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action48::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (0, 19)
    }
    fn __reduce36<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep* = EdgeStep+ => ActionFn(49);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action49::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 19)
    }
    fn __reduce37<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep+ = EdgeStep => ActionFn(61);
        let __sym0 = __pop_Variant13(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action61::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 20)
    }
    fn __reduce38<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // EdgeStep+ = EdgeStep+, EdgeStep => ActionFn(62);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant13(__symbols);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action62::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (2, 20)
    }
    fn __reduce39<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Endpoint = NodeList => ActionFn(15);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action15::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 21)
    }
    fn __reduce40<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Endpoint = Subgraph => ActionFn(16);
        let __sym0 = __pop_Variant23(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action16::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 21)
    }
    fn __reduce43<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Id? = Id => ActionFn(57);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action57::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 23)
    }
    fn __reduce44<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Id? =  => ActionFn(58);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action58::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 23)
    }
    fn __reduce45<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Kind = "graph" => ActionFn(4);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action4::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 24)
    }
    fn __reduce46<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Kind = "digraph" => ActionFn(5);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action5::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 24)
    }
    fn __reduce47<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // MacroName = Id, "=" => ActionFn(11);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action11::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 25)
    }
    fn __reduce48<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // MacroName? = MacroName => ActionFn(53);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action53::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 26)
    }
    fn __reduce49<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // MacroName? =  => ActionFn(54);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action54::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 26)
    }
    fn __reduce50<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // NodeId = Id, Port => ActionFn(102);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant17(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action102::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 27)
    }
    fn __reduce51<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // NodeId = Id => ActionFn(103);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action103::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 27)
    }
    fn __reduce52<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // NodeList = NodeId => ActionFn(73);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action73::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 28)
    }
    fn __reduce53<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // NodeList = NodeId, ("," <NodeId>)+ => ActionFn(74);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action74::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 28)
    }
    fn __reduce54<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Port = ":", Id, ":", Id => ActionFn(76);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant1(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action76::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (4, 29)
    }
    fn __reduce55<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Port = ":", Id => ActionFn(77);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action77::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (2, 29)
    }
    fn __reduce56<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Port? = Port => ActionFn(40);
        let __sym0 = __pop_Variant17(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action40::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (1, 30)
    }
    fn __reduce57<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Port? =  => ActionFn(41);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action41::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (0, 30)
    }
    fn __reduce58<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // QuotedTail = "+", "id" => ActionFn(30);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action30::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 31)
    }
    fn __reduce59<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // QuotedTail* =  => ActionFn(31);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action31::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (0, 32)
    }
    fn __reduce60<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // QuotedTail* = QuotedTail+ => ActionFn(32);
        let __sym0 = __pop_Variant19(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action32::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (1, 32)
    }
    fn __reduce61<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // QuotedTail+ = QuotedTail => ActionFn(67);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action67::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (1, 33)
    }
    fn __reduce62<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // QuotedTail+ = QuotedTail+, QuotedTail => ActionFn(68);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant19(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action68::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (2, 33)
    }
    fn __reduce63<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "graph", MacroName, AttributeList+ => ActionFn(96);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant10(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action96::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (3, 34)
    }
    fn __reduce64<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "graph", AttributeList+ => ActionFn(97);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant10(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action97::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (2, 34)
    }
    fn __reduce65<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "node", MacroName, AttributeList+ => ActionFn(98);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant10(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action98::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (3, 34)
    }
    fn __reduce66<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "node", AttributeList+ => ActionFn(99);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant10(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action99::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (2, 34)
    }
    fn __reduce67<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "edge", MacroName, AttributeList+ => ActionFn(100);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant10(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action100::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (3, 34)
    }
    fn __reduce68<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = "edge", AttributeList+ => ActionFn(101);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant10(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action101::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (2, 34)
    }
    fn __reduce69<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Statement = Id, "=", Id => ActionFn(9);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action9::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (3, 34)
    }
    fn __reduce74<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StatementItem* =  => ActionFn(55);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action55::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (0, 36)
    }
    fn __reduce75<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StatementItem* = StatementItem+ => ActionFn(56);
        let __sym0 = __pop_Variant21(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action56::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (1, 36)
    }
    fn __reduce76<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StatementItem+ = StatementItem => ActionFn(59);
        let __sym0 = __pop_Variant20(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action59::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (1, 37)
    }
    fn __reduce77<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // StatementItem+ = StatementItem+, StatementItem => ActionFn(60);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant20(__symbols);
        let __sym0 = __pop_Variant21(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action60::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (2, 37)
    }
    fn __reduce78<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Strict = "strict" => ActionFn(2);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(options, statement_count, __sym0);
        __symbols.push((__start, __Symbol::Variant22(__nt), __end));
        (1, 38)
    }
    fn __reduce79<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Strict =  => ActionFn(3);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action3::<>(options, statement_count, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant22(__nt), __end));
        (0, 38)
    }
    fn __reduce80<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "subgraph", Id, "{", "}" => ActionFn(110);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action110::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (4, 39)
    }
    fn __reduce81<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "subgraph", Id, "{", StatementItem+, "}" => ActionFn(111);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant21(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action111::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (5, 39)
    }
    fn __reduce82<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "subgraph", "{", "}" => ActionFn(112);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action112::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (3, 39)
    }
    fn __reduce83<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "subgraph", "{", StatementItem+, "}" => ActionFn(113);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant21(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action113::<>(options, statement_count, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (4, 39)
    }
    fn __reduce84<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "{", "}" => ActionFn(114);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action114::<>(options, statement_count, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (2, 39)
    }
    fn __reduce85<
        'input,
    >(
        options: &'input ParseOptions,
        statement_count: &'input Cell<usize>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Subgraph = "{", StatementItem+, "}" => ActionFn(115);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant21(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action115::<>(options, statement_count, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (3, 39)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Document::DocumentParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Document, usize),
) -> Document
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, strict, _): (usize, bool, usize),
    (_, kind, _): (usize, GraphKind, usize),
    (_, id, _): (usize, Option<Id>, usize),
    (_, _, _): (usize, Token, usize),
    (_, statements, _): (usize, alloc::vec::Vec<Statement>, usize),
    (_, _, _): (usize, Token, usize),
) -> Document
{
    Document { strict, kind, id, statements }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
) -> bool
{
    true
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> bool
{
    false
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
) -> GraphKind
{
    GraphKind::Graph
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
) -> GraphKind
{
    GraphKind::Digraph
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, _macro, _): (usize, Option<Id>, usize),
    (_, attributes, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    Statement::Attributes { scope: AttributeScope::Graph, attributes: attributes.into_iter().flatten().collect() }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, _macro, _): (usize, Option<Id>, usize),
    (_, attributes, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    Statement::Attributes { scope: AttributeScope::Node, attributes: attributes.into_iter().flatten().collect() }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, _macro, _): (usize, Option<Id>, usize),
    (_, attributes, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    Statement::Attributes { scope: AttributeScope::Edge, attributes: attributes.into_iter().flatten().collect() }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, name, _): (usize, Id, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, Id, usize),
) -> Statement
{
    Statement::Assignment { name, value }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, start, _): (usize, usize, usize),
    (_, head, _): (usize, Endpoint, usize),
    (_, steps, _): (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
    (_, attributes, _): (usize, Vec<Attribute>, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    statement_from_endpoint(head, steps, attributes, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, name, _): (usize, Id, usize),
    (_, _, _): (usize, Token, usize),
) -> Id
{
    name
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, start, _): (usize, usize, usize),
    (_, statement, _): (usize, Statement, usize),
    (_, end, _): (usize, usize, usize),
    (_, _, _): (usize, Option<Token>, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    {
    let next = statement_count.get() + 1;
    if next > options.statements {
        Err(lalrpop_util::ParseError::User { error: DotError::new(start..end, format!("DOT statement count exceeds {}", options.statements)) })
    } else {
        statement_count.set(next);
        Ok(statement)
    }
}
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, endpoint, _): (usize, Endpoint, usize),
) -> (EdgeOp, Endpoint)
{
    (EdgeOp::Directed, endpoint)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, endpoint, _): (usize, Endpoint, usize),
) -> (EdgeOp, Endpoint)
{
    (EdgeOp::Undirected, endpoint)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, nodes, _): (usize, Endpoint, usize),
) -> Endpoint
{
    nodes
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, subgraph, _): (usize, Subgraph, usize),
) -> Endpoint
{
    Endpoint::Subgraph(subgraph)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, first, _): (usize, NodeId, usize),
    (_, rest, _): (usize, alloc::vec::Vec<NodeId>, usize),
) -> Endpoint
{
    {
    if rest.is_empty() { Endpoint::Node(first) } else {
        let mut nodes = Vec::with_capacity(rest.len() + 1);
        nodes.push(first);
        nodes.extend(rest);
        Endpoint::Nodes(nodes)
    }
}
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, id, _): (usize, Id, usize),
    (_, port, _): (usize, Option<Vec<Id>>, usize),
) -> NodeId
{
    NodeId { id, port: port.unwrap_or_default() }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, first, _): (usize, Id, usize),
    (_, second, _): (usize, Option<Id>, usize),
) -> Vec<Id>
{
    {
    let mut parts = vec![first]; if let Some(second) = second { parts.push(second); } parts
}
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, id, _): (usize, Option<Id>, usize),
    (_, _, _): (usize, Token, usize),
    (_, statements, _): (usize, alloc::vec::Vec<Statement>, usize),
    (_, _, _): (usize, Token, usize),
) -> Subgraph
{
    Subgraph { id, statements }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, statements, _): (usize, alloc::vec::Vec<Statement>, usize),
    (_, _, _): (usize, Token, usize),
) -> Subgraph
{
    Subgraph { id: None, statements }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, lists, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Vec<Attribute>
{
    lists.into_iter().flatten().collect()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, entries, _): (usize, Vec<Attribute>, usize),
    (_, _, _): (usize, Token, usize),
) -> Vec<Attribute>
{
    entries
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, entries, _): (usize, alloc::vec::Vec<Attribute>, usize),
) -> Vec<Attribute>
{
    entries
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, name, _): (usize, Id, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, Id, usize),
    (_, _separator, _): (usize, (), usize),
) -> Attribute
{
    Attribute { name, value: Some(value) }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action28<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action29<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, start, _): (usize, usize, usize),
    (_, first, _): (usize, Id, usize),
    (_, tail, _): (usize, alloc::vec::Vec<Id>, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Id,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    concat_quoted(first, tail, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action30<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, next, _): (usize, Id, usize),
) -> Id
{
    next
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action31<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Id>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action32<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Id>, usize),
) -> alloc::vec::Vec<Id>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action33<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Attribute>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action34<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Attribute>, usize),
) -> alloc::vec::Vec<Attribute>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action35<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Vec<Attribute>>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action36<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> alloc::vec::Vec<Vec<Attribute>>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action37<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Id, usize),
) -> Option<Id>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action38<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Id>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action39<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, __0, _): (usize, Id, usize),
) -> Id
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action40<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Vec<Id>, usize),
) -> Option<Vec<Id>>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action41<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Vec<Id>>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action42<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<NodeId>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action43<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<NodeId>, usize),
) -> alloc::vec::Vec<NodeId>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action44<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, _, _): (usize, Token, usize),
    (_, __0, _): (usize, NodeId, usize),
) -> NodeId
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action45<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Token, usize),
) -> Option<Token>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action46<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Token>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action47<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action48<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<(EdgeOp, Endpoint)>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action49<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
) -> alloc::vec::Vec<(EdgeOp, Endpoint)>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action50<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action51<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Vec<Attribute>, usize),
) -> alloc::vec::Vec<Vec<Attribute>>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action52<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
    (_, e, _): (usize, Vec<Attribute>, usize),
) -> alloc::vec::Vec<Vec<Attribute>>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action53<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Id, usize),
) -> Option<Id>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action54<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Id>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action55<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Statement>
{
    alloc::vec![]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action56<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Statement>, usize),
) -> alloc::vec::Vec<Statement>
{
    v
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action57<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Id, usize),
) -> Option<Id>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action58<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<Id>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action59<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Statement, usize),
) -> alloc::vec::Vec<Statement>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action60<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Statement>, usize),
    (_, e, _): (usize, Statement, usize),
) -> alloc::vec::Vec<Statement>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action61<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, (EdgeOp, Endpoint), usize),
) -> alloc::vec::Vec<(EdgeOp, Endpoint)>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action62<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
    (_, e, _): (usize, (EdgeOp, Endpoint), usize),
) -> alloc::vec::Vec<(EdgeOp, Endpoint)>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action63<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, NodeId, usize),
) -> alloc::vec::Vec<NodeId>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action64<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<NodeId>, usize),
    (_, e, _): (usize, NodeId, usize),
) -> alloc::vec::Vec<NodeId>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action65<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Attribute, usize),
) -> alloc::vec::Vec<Attribute>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action66<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Attribute>, usize),
    (_, e, _): (usize, Attribute, usize),
) -> alloc::vec::Vec<Attribute>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action67<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, __0, _): (usize, Id, usize),
) -> alloc::vec::Vec<Id>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action68<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    (_, v, _): (usize, alloc::vec::Vec<Id>, usize),
    (_, e, _): (usize, Id, usize),
) -> alloc::vec::Vec<Id>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action69<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, usize, usize),
    __1: (usize, Statement, usize),
    __2: (usize, usize, usize),
    __3: (usize, Token, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action45(
        options,
        statement_count,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        options,
        statement_count,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action70<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, usize, usize),
    __1: (usize, Statement, usize),
    __2: (usize, usize, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        options,
        statement_count,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action71<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, NodeId, usize),
) -> alloc::vec::Vec<NodeId>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action44(
        options,
        statement_count,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action63(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action72<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, alloc::vec::Vec<NodeId>, usize),
    __1: (usize, Token, usize),
    __2: (usize, NodeId, usize),
) -> alloc::vec::Vec<NodeId>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action44(
        options,
        statement_count,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action64(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action73<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, NodeId, usize),
) -> Endpoint
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action42(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action74<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, NodeId, usize),
    __1: (usize, alloc::vec::Vec<NodeId>, usize),
) -> Endpoint
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action43(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action75<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
) -> Option<Id>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action39(
        options,
        statement_count,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action76<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, Token, usize),
    __3: (usize, Id, usize),
) -> Vec<Id>
{
    let __start0 = __2.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action75(
        options,
        statement_count,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action19(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action77<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
) -> Vec<Id>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action38(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action19(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action78<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
    __1: (usize, alloc::vec::Vec<Id>, usize),
    __2: (usize, usize, usize),
) -> Result<Id,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action29(
        options,
        statement_count,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action79<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Endpoint, usize),
    __1: (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
    __2: (usize, Vec<Attribute>, usize),
    __3: (usize, usize, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
        options,
        statement_count,
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
fn __action80<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Statement, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action69(
        options,
        statement_count,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action81<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Statement, usize),
    __1: (usize, usize, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action70(
        options,
        statement_count,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action82<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
    __1: (usize, alloc::vec::Vec<Id>, usize),
) -> Result<Id,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action47(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action78(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action83<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Endpoint, usize),
    __1: (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
    __2: (usize, Vec<Attribute>, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action79(
        options,
        statement_count,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action84<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Statement, usize),
    __1: (usize, Token, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action47(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action80(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action85<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Statement, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action47(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action81(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action86<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Attribute>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action33(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action24(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action87<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, alloc::vec::Vec<Attribute>, usize),
) -> Vec<Attribute>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action34(
        options,
        statement_count,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action24(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action88<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Attribute>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action35(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action89<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Vec<Attribute>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action36(
        options,
        statement_count,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        options,
        statement_count,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action90<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Endpoint, usize),
    __1: (usize, Vec<Attribute>, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action48(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action83(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action91<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Endpoint, usize),
    __1: (usize, alloc::vec::Vec<(EdgeOp, Endpoint)>, usize),
    __2: (usize, Vec<Attribute>, usize),
) -> Result<Statement,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action49(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action83(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action92<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Id, usize),
    __3: (usize, Token, usize),
    __4: (usize, alloc::vec::Vec<Statement>, usize),
    __5: (usize, Token, usize),
) -> Document
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action57(
        options,
        statement_count,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action93<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Token, usize),
    __3: (usize, alloc::vec::Vec<Statement>, usize),
    __4: (usize, Token, usize),
) -> Document
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action58(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        options,
        statement_count,
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
fn __action94<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, Token, usize),
    __3: (usize, alloc::vec::Vec<Statement>, usize),
    __4: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action57(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action20(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action95<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, alloc::vec::Vec<Statement>, usize),
    __3: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action58(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action20(
        options,
        statement_count,
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
fn __action96<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action53(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action6(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action97<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action54(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action6(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action98<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action53(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action99<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action54(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action100<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action53(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action8(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action101<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, alloc::vec::Vec<Vec<Attribute>>, usize),
) -> Statement
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action54(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action8(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action102<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
    __1: (usize, Vec<Id>, usize),
) -> NodeId
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action40(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action18(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action103<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
) -> NodeId
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action41(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action18(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action104<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
) -> Result<Id,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action31(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action82(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action105<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Id, usize),
    __1: (usize, alloc::vec::Vec<Id>, usize),
) -> Result<Id,__lalrpop_util::ParseError<usize,Token,DotError>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action32(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action82(
        options,
        statement_count,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action106<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Id, usize),
    __3: (usize, Token, usize),
    __4: (usize, Token, usize),
) -> Document
{
    let __start0 = __3.2.clone();
    let __end0 = __4.0.clone();
    let __temp0 = __action55(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action92(
        options,
        statement_count,
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
fn __action107<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Id, usize),
    __3: (usize, Token, usize),
    __4: (usize, alloc::vec::Vec<Statement>, usize),
    __5: (usize, Token, usize),
) -> Document
{
    let __start0 = __4.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action56(
        options,
        statement_count,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action92(
        options,
        statement_count,
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
fn __action108<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Document
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action55(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action93(
        options,
        statement_count,
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
fn __action109<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, bool, usize),
    __1: (usize, GraphKind, usize),
    __2: (usize, Token, usize),
    __3: (usize, alloc::vec::Vec<Statement>, usize),
    __4: (usize, Token, usize),
) -> Document
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action56(
        options,
        statement_count,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action93(
        options,
        statement_count,
        __0,
        __1,
        __2,
        __temp0,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action110<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action55(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action94(
        options,
        statement_count,
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
fn __action111<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Id, usize),
    __2: (usize, Token, usize),
    __3: (usize, alloc::vec::Vec<Statement>, usize),
    __4: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action56(
        options,
        statement_count,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action94(
        options,
        statement_count,
        __0,
        __1,
        __2,
        __temp0,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action112<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action55(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action95(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action113<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, alloc::vec::Vec<Statement>, usize),
    __3: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action56(
        options,
        statement_count,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action95(
        options,
        statement_count,
        __0,
        __1,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action114<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action55(
        options,
        statement_count,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
        options,
        statement_count,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action115<
    'input,
>(
    options: &'input ParseOptions,
    statement_count: &'input Cell<usize>,
    __0: (usize, Token, usize),
    __1: (usize, alloc::vec::Vec<Statement>, usize),
    __2: (usize, Token, usize),
) -> Subgraph
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action56(
        options,
        statement_count,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
        options,
        statement_count,
        __0,
        __temp0,
        __2,
    )
}

#[allow(clippy::type_complexity, dead_code)]
pub trait __ToTriple<'input, >
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, DotError>>;
}

impl<'input, > __ToTriple<'input, > for (usize, Token, usize)
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, DotError>> {
        Ok(self)
    }
}
impl<'input, > __ToTriple<'input, > for Result<(usize, Token, usize), DotError>
{
    fn to_triple(self) -> Result<(usize,Token,usize), __lalrpop_util::ParseError<usize, Token, DotError>> {
        self.map_err(|error| __lalrpop_util::ParseError::User { error })
    }
}
