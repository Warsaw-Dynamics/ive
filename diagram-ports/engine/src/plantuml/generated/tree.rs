// auto-generated: "lalrpop 0.23.2"
// sha3: 486ad2438703388a923d1cea8e7c3eb2e93fcb4d0e66762a1b6d945d88773c1a
// features: 
use crate::plantuml::{tree::{Outline, Side}, parser::{tree::{Token, Marker, Direction, marker, alias}, error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Mindmap {

    use crate::plantuml::{tree::{Outline, Side}, parser::{tree::{Token, Marker, Direction, marker, alias}, error::ParseFailure}};
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
        Variant0(String),
        Variant1(Token),
        Variant2(Side),
        Variant3(usize),
        Variant4((String, String)),
        Variant5(()),
        Variant6((Outline, Direction)),
        Variant7(Outline),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -11, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11, 0, 0,
        // State 1
        6, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        -12, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, 0, 0,
        // State 4
        0, 0, 9, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0,
        // State 5
        0, 0, -13, 0, 0, 0, 0, 0, 0, -13, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0,
        // State 7
        0, 0, -14, 0, 0, 0, 0, 0, 0, -14, 0, 0, 0, 0, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0,
        // State 10
        -8, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0,
        // State 12
        -9, -9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -9, 0, 0,
        // State 13
        -10, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -10, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 15 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -11,
        // State 1
        -15,
        // State 2
        -78,
        // State 3
        -12,
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
        -8,
        // State 11
        0,
        // State 12
        -9,
        // State 13
        -10,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            4 => 3,
            5 => 1,
            6 => 4,
            7 => 2,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""prefix""###,
        r###""zero""###,
        r###""_""###,
        r###""<""###,
        r###"">""###,
        r###""(""###,
        r###"")""###,
        r###""alias""###,
        r###""space""###,
        r###""label""###,
        r###""quoted""###,
        r###""as""###,
        r###""side""###,
        r###""newline""###,
        r###""unknown""###,
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
    >(
        __states: &[i8],
        _: core::marker::PhantomData<()>,
    ) -> alloc::vec::Vec<alloc::string::String>
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<()>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'__0>
    where 
    {
        builder: &'__0 mut dyn crate::plantuml::parser::tree::Actions,
        __phantom: core::marker::PhantomData<()>,
    }
    impl<'__0> __state_machine::ParserDefinition for __StateMachine<'__0>
    where 
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
            __action(state, 15 - 1)
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
            Token::Prefix(_) if true => Some(0),
            Token::Zero if true => Some(1),
            Token::Under if true => Some(2),
            Token::Left if true => Some(3),
            Token::Right if true => Some(4),
            Token::Open if true => Some(5),
            Token::Close if true => Some(6),
            Token::Alias(_) if true => Some(7),
            Token::Space if true => Some(8),
            Token::Label(_) if true => Some(9),
            Token::Quoted(_) if true => Some(10),
            Token::As if true => Some(11),
            Token::Side(_) if true => Some(12),
            Token::Newline if true => Some(13),
            Token::Unknown if true => Some(14),
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
            0 | 7 | 9 | 10 => match __token {
                Token::Prefix(__tok0) | Token::Alias(__tok0) | Token::Label(__tok0) | Token::Quoted(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            1 | 2 | 3 | 4 | 5 | 6 | 8 | 11 | 13 | 14 => __Symbol::Variant1(__token),
            12 => match __token {
                Token::Side(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        '__0,
    >(
        __reduce_index: i8,
        _: core::marker::PhantomData<()>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'__0>>
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
                    states_to_pop: 0,
                    nonterminal_produced: 2,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
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
                    states_to_pop: 4,
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
                    states_to_pop: 3,
                    nonterminal_produced: 4,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 4,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 5,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
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
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 10,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
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
                    states_to_pop: 2,
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
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            44 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            45 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            46 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            47 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            48 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            49 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            50 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            51 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            52 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            53 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            54 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            55 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            56 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            57 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            58 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            59 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            60 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            61 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            62 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            63 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            64 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            65 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            66 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            67 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            68 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            69 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            70 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            71 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            72 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            73 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            74 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            75 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            76 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 13,
                }
            }
            77 => __state_machine::SimulatedReduce::Accept,
            78 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 15,
                }
            }
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct MindmapParser {
        _priv: (),
    }

    impl Default for MindmapParser { fn default() -> Self { Self::new() } }
    impl MindmapParser {
        pub fn new() -> MindmapParser {
            MindmapParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut dyn crate::plantuml::parser::tree::Actions,
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
    >(
        __error_state: Option<i8>,
        __states: &[i8],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<()>,
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
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
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
                // MindItem = "side", "newline" => ActionFn(5);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action5::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 4)
            }
            8 => {
                // MindItem = MindPrefix, "label", "newline" => ActionFn(42);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action42::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 4)
            }
            9 => {
                // MindItem = MindPrefix, "_", "label", "newline" => ActionFn(43);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action43::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 4)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            11 => {
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                __reduce19(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                // Name = "alias" => ActionFn(32);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action32::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant0(__nt), __end));
                (1, 9)
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
                // WbsItem = "prefix", "newline" => ActionFn(50);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action50::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 12)
            }
            28 => {
                // WbsItem = "prefix", "_", "newline" => ActionFn(51);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action51::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            29 => {
                // WbsItem = "prefix", "<", "newline" => ActionFn(52);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action52::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            30 => {
                // WbsItem = "prefix", "_", "<", "newline" => ActionFn(53);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action53::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            31 => {
                // WbsItem = "prefix", ">", "newline" => ActionFn(54);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action54::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            32 => {
                // WbsItem = "prefix", "_", ">", "newline" => ActionFn(55);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action55::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            33 => {
                // WbsItem = "prefix", "<", "_", "newline" => ActionFn(56);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action56::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            34 => {
                // WbsItem = "prefix", ">", "_", "newline" => ActionFn(57);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action57::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            35 => {
                // WbsItem = "prefix", "space", "label", "newline" => ActionFn(58);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action58::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            36 => {
                // WbsItem = "prefix", "_", "space", "label", "newline" => ActionFn(59);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action59::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            37 => {
                // WbsItem = "prefix", "<", "space", "label", "newline" => ActionFn(60);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action60::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            38 => {
                // WbsItem = "prefix", "_", "<", "space", "label", "newline" => ActionFn(61);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action61::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            39 => {
                // WbsItem = "prefix", ">", "space", "label", "newline" => ActionFn(62);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action62::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            40 => {
                // WbsItem = "prefix", "_", ">", "space", "label", "newline" => ActionFn(63);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action63::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            41 => {
                // WbsItem = "prefix", "<", "_", "space", "label", "newline" => ActionFn(64);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action64::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            42 => {
                // WbsItem = "prefix", ">", "_", "space", "label", "newline" => ActionFn(65);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action65::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            43 => {
                // WbsItem = "prefix", "space", "quoted", "as", Name, "newline" => ActionFn(66);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action66::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            44 => {
                // WbsItem = "prefix", "_", "space", "quoted", "as", Name, "newline" => ActionFn(67);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action67::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            45 => {
                // WbsItem = "prefix", "<", "space", "quoted", "as", Name, "newline" => ActionFn(68);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action68::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            46 => {
                // WbsItem = "prefix", "_", "<", "space", "quoted", "as", Name, "newline" => ActionFn(69);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action69::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            47 => {
                // WbsItem = "prefix", ">", "space", "quoted", "as", Name, "newline" => ActionFn(70);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action70::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            48 => {
                // WbsItem = "prefix", "_", ">", "space", "quoted", "as", Name, "newline" => ActionFn(71);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action71::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            49 => {
                // WbsItem = "prefix", "<", "_", "space", "quoted", "as", Name, "newline" => ActionFn(72);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action72::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            50 => {
                // WbsItem = "prefix", ">", "_", "space", "quoted", "as", Name, "newline" => ActionFn(73);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action73::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            51 => {
                // WbsItem = "prefix", "(", Name, ")", "newline" => ActionFn(74);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action74::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            52 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "newline" => ActionFn(75);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action75::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            53 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "newline" => ActionFn(76);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action76::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            54 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "newline" => ActionFn(77);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action77::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            55 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "newline" => ActionFn(78);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action78::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            56 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "newline" => ActionFn(79);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action79::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            57 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "newline" => ActionFn(80);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action80::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            58 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "newline" => ActionFn(81);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action81::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            59 => {
                // WbsItem = "prefix", "(", Name, ")", "space", "label", "newline" => ActionFn(82);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action82::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            60 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(83);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action83::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            61 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "space", "label", "newline" => ActionFn(84);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action84::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            62 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "space", "label", "newline" => ActionFn(85);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action85::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            63 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "space", "label", "newline" => ActionFn(86);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action86::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            64 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "space", "label", "newline" => ActionFn(87);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action87::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            65 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(88);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action88::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            66 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(89);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action89::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            67 => {
                // WbsItem = "prefix", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(90);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action90::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            68 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(91);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action91::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            69 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(92);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action92::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            70 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(93);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action93::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            71 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(94);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action94::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            72 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(95);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action95::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            73 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(96);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action96::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            74 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(97);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action97::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            75 => {
                __reduce75(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            76 => {
                __reduce76(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            77 => {
                // __Mindmap = Mindmap => ActionFn(0);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(builder, __sym0);
                return Some(Ok(__nt));
            }
            78 => {
                __reduce78(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (Outline, Direction), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (String, String), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Outline, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Side, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Token, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
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
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(27);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action27::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(26);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action26::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 1)
    }
    fn __reduce2<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Alias =  => ActionFn(20);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action20::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (0, 2)
    }
    fn __reduce3<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Alias = "(", Name, ")" => ActionFn(21);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action21::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (3, 2)
    }
    fn __reduce4<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label =  => ActionFn(23);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action23::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 3)
    }
    fn __reduce5<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label = "space", "label" => ActionFn(24);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action24::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 3)
    }
    fn __reduce6<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label = "space", "quoted", "as", Name => ActionFn(25);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action25::<>(builder, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (4, 3)
    }
    fn __reduce10<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindItems =  => ActionFn(3);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action3::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
    }
    fn __reduce11<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindItems = MindItems, MindItem => ActionFn(4);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action4::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 5)
    }
    fn __reduce12<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindPrefix = "prefix" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 6)
    }
    fn __reduce13<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindPrefix = "zero" => ActionFn(8);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 6)
    }
    fn __reduce14<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Mindmap = MindItems => ActionFn(2);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 7)
    }
    fn __reduce15<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers =  => ActionFn(44);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action44::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 8)
    }
    fn __reduce16<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_" => ActionFn(45);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce17<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "<" => ActionFn(46);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action46::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce18<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_", "<" => ActionFn(47);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action47::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce19<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = ">" => ActionFn(48);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action48::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce20<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_", ">" => ActionFn(49);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action49::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce21<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "<", "_" => ActionFn(18);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action18::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce22<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = ">", "_" => ActionFn(19);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action19::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce24<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Shape =  => ActionFn(9);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action9::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (0, 10)
    }
    fn __reduce25<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Shape = "_" => ActionFn(10);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
    }
    fn __reduce26<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Wbs = WbsItems => ActionFn(11);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 11)
    }
    fn __reduce75<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // WbsItems =  => ActionFn(12);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action12::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 13)
    }
    fn __reduce76<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // WbsItems = WbsItems, WbsItem => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 13)
    }
    fn __reduce78<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // __Wbs = Wbs => ActionFn(1);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 15)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Mindmap::MindmapParser;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Wbs {

    use crate::plantuml::{tree::{Outline, Side}, parser::{tree::{Token, Marker, Direction, marker, alias}, error::ParseFailure}};
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
        Variant0(String),
        Variant1(Token),
        Variant2(Side),
        Variant3(usize),
        Variant4((String, String)),
        Variant5(()),
        Variant6((Outline, Direction)),
        Variant7(Outline),
    }
    const __ACTION: &[i16] = &[
        // State 0
        -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 22
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 27
        -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 28
        0, 0, 32, 30, 31, 3, 0, 0, 34, 0, 0, 0, 0, 33, 0,
        // State 29
        0, 0, 37, 0, 0, 4, 0, 0, 39, 0, 0, 0, 0, 38, 0,
        // State 30
        0, 0, 40, 0, 0, 5, 0, 0, 42, 0, 0, 0, 0, 41, 0,
        // State 31
        0, 0, 0, 43, 44, 6, 0, 0, 46, 0, 0, 0, 0, 45, 0,
        // State 32
        -28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 48, 0, 0, 0, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 49, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, -24, 0, 0, 0, 0, 0, 0, -24, 0,
        // State 36
        0, 0, 0, 0, 0, 7, 0, 0, 52, 0, 0, 0, 0, 51, 0,
        // State 37
        -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 54, 0, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 8, 0, 0, 57, 0, 0, 0, 0, 56, 0,
        // State 40
        -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 59, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 9, 0, 0, 62, 0, 0, 0, 0, 61, 0,
        // State 43
        0, 0, 0, 0, 0, 10, 0, 0, 64, 0, 0, 0, 0, 63, 0,
        // State 44
        -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 66, 0, 0, 0, 0,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 67, 0,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 69, 0, 0, 0, 0, 68, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 70, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 50
        -34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 51
        0, 0, 0, 0, 0, 0, 0, 0, 0, 72, 73, 0, 0, 0, 0,
        // State 52
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 0,
        // State 53
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0,
        // State 54
        0, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 55
        -35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 56
        0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 78, 0, 0, 0, 0,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 79, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 60
        -31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 61
        0, 0, 0, 0, 0, 0, 0, 0, 0, 82, 83, 0, 0, 0, 0,
        // State 62
        -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 63
        0, 0, 0, 0, 0, 0, 0, 0, 0, 85, 86, 0, 0, 0, 0,
        // State 64
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 87, 0,
        // State 65
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0,
        // State 66
        -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 67
        -52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 68
        0, 0, 0, 0, 0, 0, 0, 0, 0, 89, 90, 0, 0, 0, 0,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 0, 91, 0,
        // State 70
        0, 0, 0, 0, 0, 0, 93, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 71
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 0,
        // State 72
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0,
        // State 73
        -38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 74
        0, 0, 0, 0, 0, 0, 0, 0, 97, 0, 0, 0, 0, 96, 0,
        // State 75
        0, 0, 0, 0, 0, 0, 98, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99, 0,
        // State 77
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
        // State 78
        -40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 79
        0, 0, 0, 0, 0, 0, 0, 0, 102, 0, 0, 0, 0, 101, 0,
        // State 80
        0, 0, 0, 0, 0, 0, 103, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 81
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 0,
        // State 82
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0,
        // State 83
        0, 0, 0, 0, 0, 0, 105, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 84
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 0,
        // State 85
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0,
        // State 86
        -37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 87
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 108, 0,
        // State 88
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 109, 0,
        // State 89
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0,
        // State 90
        -54, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 91
        0, 0, 0, 0, 0, 0, 0, 0, 0, 110, 111, 0, 0, 0, 0,
        // State 92
        0, 0, 0, 0, 0, 0, 0, 0, 113, 0, 0, 0, 0, 112, 0,
        // State 93
        -42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 94
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 115, 0,
        // State 95
        -56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 96
        0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 117, 0, 0, 0, 0,
        // State 97
        0, 0, 0, 0, 0, 0, 0, 0, 119, 0, 0, 0, 0, 118, 0,
        // State 98
        -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 99
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 0,
        // State 100
        -53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 101
        0, 0, 0, 0, 0, 0, 0, 0, 0, 122, 123, 0, 0, 0, 0,
        // State 102
        0, 0, 0, 0, 0, 0, 0, 0, 125, 0, 0, 0, 0, 124, 0,
        // State 103
        -39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 104
        0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 127, 0,
        // State 105
        -41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 106
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 130, 0,
        // State 107
        -44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 108
        -60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 109
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 132, 0,
        // State 110
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0,
        // State 111
        -58, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 112
        0, 0, 0, 0, 0, 0, 0, 0, 0, 133, 134, 0, 0, 0, 0,
        // State 113
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 135, 0,
        // State 114
        -46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 115
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 0,
        // State 116
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 0, 0,
        // State 117
        -59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 118
        0, 0, 0, 0, 0, 0, 0, 0, 0, 137, 138, 0, 0, 0, 0,
        // State 119
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 139, 0,
        // State 120
        -48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 121
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 140, 0,
        // State 122
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 0, 0,
        // State 123
        -55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 124
        0, 0, 0, 0, 0, 0, 0, 0, 0, 141, 142, 0, 0, 0, 0,
        // State 125
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 143, 0,
        // State 126
        -57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 127
        0, 0, 0, 0, 0, 0, 0, 0, 0, 144, 145, 0, 0, 0, 0,
        // State 128
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 146, 0,
        // State 129
        -45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 130
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 147, 0,
        // State 131
        -62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 132
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 149, 0,
        // State 133
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 0, 0, 0,
        // State 134
        -50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 135
        -64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 136
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 151, 0,
        // State 137
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 0, 0, 0,
        // State 138
        -51, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 139
        -61, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 140
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 153, 0,
        // State 141
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0,
        // State 142
        -47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 143
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 154, 0,
        // State 144
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 0, 0, 0,
        // State 145
        -49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 146
        -68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 147
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 155, 0,
        // State 148
        -66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 149
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 157, 0,
        // State 150
        -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 151
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 159, 0,
        // State 152
        -63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 153
        -65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 154
        -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 155
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 162, 0,
        // State 156
        -72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 157
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 163, 0,
        // State 158
        -69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 159
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 164, 0,
        // State 160
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 165, 0,
        // State 161
        -74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 162
        -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 163
        -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 164
        -73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    fn __action(state: i16, integer: usize) -> i16 {
        __ACTION[(state as usize) * 15 + integer]
    }
    const __EOF_ACTION: &[i16] = &[
        // State 0
        -76,
        // State 1
        -27,
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
        -79,
        // State 27
        -77,
        // State 28
        0,
        // State 29
        0,
        // State 30
        0,
        // State 31
        0,
        // State 32
        -28,
        // State 33
        0,
        // State 34
        0,
        // State 35
        0,
        // State 36
        0,
        // State 37
        -30,
        // State 38
        0,
        // State 39
        0,
        // State 40
        -32,
        // State 41
        0,
        // State 42
        0,
        // State 43
        0,
        // State 44
        -29,
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
        -34,
        // State 51
        0,
        // State 52
        0,
        // State 53
        0,
        // State 54
        0,
        // State 55
        -35,
        // State 56
        0,
        // State 57
        0,
        // State 58
        0,
        // State 59
        0,
        // State 60
        -31,
        // State 61
        0,
        // State 62
        -33,
        // State 63
        0,
        // State 64
        0,
        // State 65
        0,
        // State 66
        -36,
        // State 67
        -52,
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
        -38,
        // State 74
        0,
        // State 75
        0,
        // State 76
        0,
        // State 77
        0,
        // State 78
        -40,
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
        -37,
        // State 87
        0,
        // State 88
        0,
        // State 89
        0,
        // State 90
        -54,
        // State 91
        0,
        // State 92
        0,
        // State 93
        -42,
        // State 94
        0,
        // State 95
        -56,
        // State 96
        0,
        // State 97
        0,
        // State 98
        -43,
        // State 99
        0,
        // State 100
        -53,
        // State 101
        0,
        // State 102
        0,
        // State 103
        -39,
        // State 104
        0,
        // State 105
        -41,
        // State 106
        0,
        // State 107
        -44,
        // State 108
        -60,
        // State 109
        0,
        // State 110
        0,
        // State 111
        -58,
        // State 112
        0,
        // State 113
        0,
        // State 114
        -46,
        // State 115
        0,
        // State 116
        0,
        // State 117
        -59,
        // State 118
        0,
        // State 119
        0,
        // State 120
        -48,
        // State 121
        0,
        // State 122
        0,
        // State 123
        -55,
        // State 124
        0,
        // State 125
        0,
        // State 126
        -57,
        // State 127
        0,
        // State 128
        0,
        // State 129
        -45,
        // State 130
        0,
        // State 131
        -62,
        // State 132
        0,
        // State 133
        0,
        // State 134
        -50,
        // State 135
        -64,
        // State 136
        0,
        // State 137
        0,
        // State 138
        -51,
        // State 139
        -61,
        // State 140
        0,
        // State 141
        0,
        // State 142
        -47,
        // State 143
        0,
        // State 144
        0,
        // State 145
        -49,
        // State 146
        -68,
        // State 147
        0,
        // State 148
        -66,
        // State 149
        0,
        // State 150
        -67,
        // State 151
        0,
        // State 152
        -63,
        // State 153
        -65,
        // State 154
        -70,
        // State 155
        0,
        // State 156
        -72,
        // State 157
        0,
        // State 158
        -69,
        // State 159
        0,
        // State 160
        0,
        // State 161
        -74,
        // State 162
        -75,
        // State 163
        -71,
        // State 164
        -73,
    ];
    fn __goto(state: i16, nt: usize) -> i16 {
        match nt {
            9 => match state {
                3 => 49,
                4 => 54,
                5 => 59,
                6 => 70,
                7 => 75,
                8 => 80,
                9 => 83,
                10 => 87,
                11 => 94,
                12 => 99,
                13 => 106,
                14 => 113,
                15 => 119,
                16 => 125,
                17 => 128,
                18 => 130,
                19 => 147,
                20 => 149,
                21 => 151,
                22 => 155,
                23 => 157,
                24 => 159,
                25 => 160,
                _ => 34,
            },
            11 => 26,
            12 => 27,
            13 => 1,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""prefix""###,
        r###""zero""###,
        r###""_""###,
        r###""<""###,
        r###"">""###,
        r###""(""###,
        r###"")""###,
        r###""alias""###,
        r###""space""###,
        r###""label""###,
        r###""quoted""###,
        r###""as""###,
        r###""side""###,
        r###""newline""###,
        r###""unknown""###,
    ];
    fn __expected_tokens(__state: i16) -> alloc::vec::Vec<alloc::string::String> {
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
    >(
        __states: &[i16],
        _: core::marker::PhantomData<()>,
    ) -> alloc::vec::Vec<alloc::string::String>
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<()>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'__0>
    where 
    {
        builder: &'__0 mut dyn crate::plantuml::parser::tree::Actions,
        __phantom: core::marker::PhantomData<()>,
    }
    impl<'__0> __state_machine::ParserDefinition for __StateMachine<'__0>
    where 
    {
        type Location = usize;
        type Error = ParseFailure;
        type Token = Token;
        type TokenIndex = usize;
        type Symbol = __Symbol<>;
        type Success = ();
        type StateIndex = i16;
        type Action = i16;
        type ReduceIndex = i16;
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
        fn action(&self, state: i16, integer: usize) -> i16 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i16) -> i16 {
            __action(state, 15 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i16) -> i16 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i16, nt: usize) -> i16 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<()>)
        }

        fn expected_tokens(&self, state: i16) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens(state)
        }

        fn expected_tokens_from_states(&self, states: &[i16]) -> alloc::vec::Vec<alloc::string::String> {
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
            action: i16,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i16>,
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

        fn simulate_reduce(&self, action: i16) -> __state_machine::SimulatedReduce<Self> {
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
            Token::Prefix(_) if true => Some(0),
            Token::Zero if true => Some(1),
            Token::Under if true => Some(2),
            Token::Left if true => Some(3),
            Token::Right if true => Some(4),
            Token::Open if true => Some(5),
            Token::Close if true => Some(6),
            Token::Alias(_) if true => Some(7),
            Token::Space if true => Some(8),
            Token::Label(_) if true => Some(9),
            Token::Quoted(_) if true => Some(10),
            Token::As if true => Some(11),
            Token::Side(_) if true => Some(12),
            Token::Newline if true => Some(13),
            Token::Unknown if true => Some(14),
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
            0 | 7 | 9 | 10 => match __token {
                Token::Prefix(__tok0) | Token::Alias(__tok0) | Token::Label(__tok0) | Token::Quoted(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            1 | 2 | 3 | 4 | 5 | 6 | 8 | 11 | 13 | 14 => __Symbol::Variant1(__token),
            12 => match __token {
                Token::Side(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        '__0,
    >(
        __reduce_index: i16,
        _: core::marker::PhantomData<()>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'__0>>
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
                    states_to_pop: 0,
                    nonterminal_produced: 2,
                }
            }
            3 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            4 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
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
                    states_to_pop: 4,
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
                    states_to_pop: 3,
                    nonterminal_produced: 4,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 4,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 5,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
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
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 8,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 10,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
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
                    states_to_pop: 2,
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
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            30 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            31 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 12,
                }
            }
            32 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            33 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            34 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            35 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 12,
                }
            }
            36 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            37 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            38 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            39 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            40 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            41 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            42 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            43 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            44 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            45 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            46 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            47 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            48 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            49 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            50 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            51 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 12,
                }
            }
            52 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            53 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            54 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            55 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 12,
                }
            }
            56 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            57 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            58 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            59 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 12,
                }
            }
            60 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            61 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            62 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            63 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 8,
                    nonterminal_produced: 12,
                }
            }
            64 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            65 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            66 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            67 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 9,
                    nonterminal_produced: 12,
                }
            }
            68 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            69 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            70 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            71 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 10,
                    nonterminal_produced: 12,
                }
            }
            72 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            73 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            74 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 11,
                    nonterminal_produced: 12,
                }
            }
            75 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 13,
                }
            }
            76 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 13,
                }
            }
            77 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 14,
                }
            }
            78 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct WbsParser {
        _priv: (),
    }

    impl Default for WbsParser { fn default() -> Self { Self::new() } }
    impl WbsParser {
        pub fn new() -> WbsParser {
            WbsParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut dyn crate::plantuml::parser::tree::Actions,
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
    >(
        __error_state: Option<i16>,
        __states: &[i16],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<()>,
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
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __action: i16,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i16>,
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
                // MindItem = "side", "newline" => ActionFn(5);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action5::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 4)
            }
            8 => {
                // MindItem = MindPrefix, "label", "newline" => ActionFn(42);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action42::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 4)
            }
            9 => {
                // MindItem = MindPrefix, "_", "label", "newline" => ActionFn(43);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action43::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 4)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            11 => {
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                __reduce19(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                // Name = "alias" => ActionFn(32);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action32::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant0(__nt), __end));
                (1, 9)
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
                // WbsItem = "prefix", "newline" => ActionFn(50);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action50::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 12)
            }
            28 => {
                // WbsItem = "prefix", "_", "newline" => ActionFn(51);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action51::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            29 => {
                // WbsItem = "prefix", "<", "newline" => ActionFn(52);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action52::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            30 => {
                // WbsItem = "prefix", "_", "<", "newline" => ActionFn(53);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action53::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            31 => {
                // WbsItem = "prefix", ">", "newline" => ActionFn(54);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action54::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 12)
            }
            32 => {
                // WbsItem = "prefix", "_", ">", "newline" => ActionFn(55);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action55::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            33 => {
                // WbsItem = "prefix", "<", "_", "newline" => ActionFn(56);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action56::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            34 => {
                // WbsItem = "prefix", ">", "_", "newline" => ActionFn(57);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action57::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            35 => {
                // WbsItem = "prefix", "space", "label", "newline" => ActionFn(58);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action58::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (4, 12)
            }
            36 => {
                // WbsItem = "prefix", "_", "space", "label", "newline" => ActionFn(59);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action59::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            37 => {
                // WbsItem = "prefix", "<", "space", "label", "newline" => ActionFn(60);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action60::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            38 => {
                // WbsItem = "prefix", "_", "<", "space", "label", "newline" => ActionFn(61);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action61::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            39 => {
                // WbsItem = "prefix", ">", "space", "label", "newline" => ActionFn(62);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action62::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            40 => {
                // WbsItem = "prefix", "_", ">", "space", "label", "newline" => ActionFn(63);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action63::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            41 => {
                // WbsItem = "prefix", "<", "_", "space", "label", "newline" => ActionFn(64);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action64::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            42 => {
                // WbsItem = "prefix", ">", "_", "space", "label", "newline" => ActionFn(65);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action65::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            43 => {
                // WbsItem = "prefix", "space", "quoted", "as", Name, "newline" => ActionFn(66);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action66::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            44 => {
                // WbsItem = "prefix", "_", "space", "quoted", "as", Name, "newline" => ActionFn(67);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action67::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            45 => {
                // WbsItem = "prefix", "<", "space", "quoted", "as", Name, "newline" => ActionFn(68);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action68::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            46 => {
                // WbsItem = "prefix", "_", "<", "space", "quoted", "as", Name, "newline" => ActionFn(69);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action69::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            47 => {
                // WbsItem = "prefix", ">", "space", "quoted", "as", Name, "newline" => ActionFn(70);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action70::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            48 => {
                // WbsItem = "prefix", "_", ">", "space", "quoted", "as", Name, "newline" => ActionFn(71);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action71::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            49 => {
                // WbsItem = "prefix", "<", "_", "space", "quoted", "as", Name, "newline" => ActionFn(72);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action72::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            50 => {
                // WbsItem = "prefix", ">", "_", "space", "quoted", "as", Name, "newline" => ActionFn(73);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action73::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            51 => {
                // WbsItem = "prefix", "(", Name, ")", "newline" => ActionFn(74);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action74::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (5, 12)
            }
            52 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "newline" => ActionFn(75);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action75::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            53 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "newline" => ActionFn(76);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action76::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            54 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "newline" => ActionFn(77);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action77::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            55 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "newline" => ActionFn(78);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action78::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (6, 12)
            }
            56 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "newline" => ActionFn(79);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action79::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            57 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "newline" => ActionFn(80);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action80::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            58 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "newline" => ActionFn(81);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action81::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            59 => {
                // WbsItem = "prefix", "(", Name, ")", "space", "label", "newline" => ActionFn(82);
                assert!(__symbols.len() >= 7);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym6.2.clone();
                let __nt = match super::__action82::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (7, 12)
            }
            60 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(83);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action83::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            61 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "space", "label", "newline" => ActionFn(84);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action84::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            62 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "space", "label", "newline" => ActionFn(85);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action85::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            63 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "space", "label", "newline" => ActionFn(86);
                assert!(__symbols.len() >= 8);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym7.2.clone();
                let __nt = match super::__action86::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (8, 12)
            }
            64 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "space", "label", "newline" => ActionFn(87);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action87::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            65 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(88);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action88::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            66 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "space", "label", "newline" => ActionFn(89);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action89::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            67 => {
                // WbsItem = "prefix", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(90);
                assert!(__symbols.len() >= 9);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant0(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym8.2.clone();
                let __nt = match super::__action90::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (9, 12)
            }
            68 => {
                // WbsItem = "prefix", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(91);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action91::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            69 => {
                // WbsItem = "prefix", "<", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(92);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action92::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            70 => {
                // WbsItem = "prefix", "_", "<", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(93);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action93::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            71 => {
                // WbsItem = "prefix", ">", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(94);
                assert!(__symbols.len() >= 10);
                let __sym9 = __pop_Variant1(__symbols);
                let __sym8 = __pop_Variant0(__symbols);
                let __sym7 = __pop_Variant1(__symbols);
                let __sym6 = __pop_Variant0(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym9.2.clone();
                let __nt = match super::__action94::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (10, 12)
            }
            72 => {
                // WbsItem = "prefix", "_", ">", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(95);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action95::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            73 => {
                // WbsItem = "prefix", "<", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(96);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action96::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            74 => {
                // WbsItem = "prefix", ">", "_", "(", Name, ")", "space", "quoted", "as", Name, "newline" => ActionFn(97);
                assert!(__symbols.len() >= 11);
                let __sym10 = __pop_Variant1(__symbols);
                let __sym9 = __pop_Variant0(__symbols);
                let __sym8 = __pop_Variant1(__symbols);
                let __sym7 = __pop_Variant0(__symbols);
                let __sym6 = __pop_Variant1(__symbols);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym10.2.clone();
                let __nt = match super::__action97::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (11, 12)
            }
            75 => {
                __reduce75(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            76 => {
                __reduce76(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            77 => {
                __reduce77(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            78 => {
                // __Wbs = Wbs => ActionFn(1);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action1::<>(builder, __sym0);
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
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (Outline, Direction), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (String, String), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Outline, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Side, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Token, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
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
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(27);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action27::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(26);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action26::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 1)
    }
    fn __reduce2<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Alias =  => ActionFn(20);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action20::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (0, 2)
    }
    fn __reduce3<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Alias = "(", Name, ")" => ActionFn(21);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action21::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (3, 2)
    }
    fn __reduce4<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label =  => ActionFn(23);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action23::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 3)
    }
    fn __reduce5<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label = "space", "label" => ActionFn(24);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action24::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 3)
    }
    fn __reduce6<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Label = "space", "quoted", "as", Name => ActionFn(25);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action25::<>(builder, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (4, 3)
    }
    fn __reduce10<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindItems =  => ActionFn(3);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action3::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
    }
    fn __reduce11<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindItems = MindItems, MindItem => ActionFn(4);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action4::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 5)
    }
    fn __reduce12<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindPrefix = "prefix" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 6)
    }
    fn __reduce13<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // MindPrefix = "zero" => ActionFn(8);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 6)
    }
    fn __reduce14<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Mindmap = MindItems => ActionFn(2);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 7)
    }
    fn __reduce15<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers =  => ActionFn(44);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action44::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 8)
    }
    fn __reduce16<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_" => ActionFn(45);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce17<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "<" => ActionFn(46);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action46::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce18<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_", "<" => ActionFn(47);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action47::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce19<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = ">" => ActionFn(48);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action48::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 8)
    }
    fn __reduce20<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "_", ">" => ActionFn(49);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action49::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce21<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = "<", "_" => ActionFn(18);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action18::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce22<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Modifiers = ">", "_" => ActionFn(19);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action19::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    fn __reduce24<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Shape =  => ActionFn(9);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action9::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (0, 10)
    }
    fn __reduce25<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Shape = "_" => ActionFn(10);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
    }
    fn __reduce26<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Wbs = WbsItems => ActionFn(11);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 11)
    }
    fn __reduce75<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // WbsItems =  => ActionFn(12);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action12::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 13)
    }
    fn __reduce76<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // WbsItems = WbsItems, WbsItem => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 13)
    }
    fn __reduce77<
    >(
        builder: &mut dyn crate::plantuml::parser::tree::Actions,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // __Mindmap = Mindmap => ActionFn(0);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action0::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 14)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Wbs::WbsParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, side, _): (usize, Side, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.set_side(side).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, start, _): (usize, usize, usize),
    (_, prefix, _): (usize, String, usize),
    (_, shape, _): (usize, Outline, usize),
    (_, label, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.node(marker(prefix, (shape, Direction::Default), String::new(), (label, String::new())), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, prefix, _): (usize, String, usize),
) -> String
{
    prefix
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, Token, usize),
) -> String
{
    "0".into()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Outline
{
    Outline::Box
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, Token, usize),
) -> Outline
{
    Outline::Boxless
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, start, _): (usize, usize, usize),
    (_, prefix, _): (usize, String, usize),
    (_, modifiers, _): (usize, (Outline, Direction), usize),
    (_, name, _): (usize, String, usize),
    (_, label, _): (usize, (String, String), usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.node(marker(prefix, modifiers, name, label), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, shape, _): (usize, Outline, usize),
) -> (Outline, Direction)
{
    (shape, Direction::Default)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, shape, _): (usize, Outline, usize),
    (_, _, _): (usize, Token, usize),
) -> (Outline, Direction)
{
    (shape, Direction::Left)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, shape, _): (usize, Outline, usize),
    (_, _, _): (usize, Token, usize),
) -> (Outline, Direction)
{
    (shape, Direction::Right)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
) -> (Outline, Direction)
{
    (Outline::Boxless, Direction::Left)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, __0, _): (usize, Token, usize),
    (_, __1, _): (usize, Token, usize),
) -> (Outline, Direction)
{
    (Outline::Boxless, Direction::Right)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    alias(value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> (String, String)
{
    (String::new(), String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, _, _): (usize, Token, usize),
    (_, label, _): (usize, String, usize),
) -> (String, String)
{
    (label, String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    (_, _, _): (usize, Token, usize),
    (_, display, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, name, _): (usize, String, usize),
) -> (String, String)
{
    (display, name)
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action26<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action27<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action28<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Outline, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action27(
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
fn __action29<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action27(
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
fn __action30<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, String, usize),
    __3: (usize, (String, String), usize),
    __4: (usize, Token, usize),
    __5: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action27(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action14(
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
fn __action31<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Outline, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
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
fn __action32<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action29(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action33<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, String, usize),
    __3: (usize, (String, String), usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action30(
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
fn __action34<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, (String, String), usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action20(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action33(
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
fn __action35<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, (String, String), usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.0.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action21(
        builder,
        __2,
        __3,
        __4,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action33(
        builder,
        __0,
        __1,
        __temp0,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action36<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action23(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
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
fn __action37<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action24(
        builder,
        __2,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        builder,
        __0,
        __1,
        __temp0,
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action38<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.0.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action25(
        builder,
        __2,
        __3,
        __4,
        __5,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        builder,
        __0,
        __1,
        __temp0,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action39<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __4.2.clone();
    let __end0 = __5.0.clone();
    let __temp0 = __action23(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action40<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.0.clone();
    let __end0 = __6.2.clone();
    let __temp0 = __action24(
        builder,
        __5,
        __6,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action41<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, (Outline, Direction), usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
    __8: (usize, String, usize),
    __9: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.0.clone();
    let __end0 = __8.2.clone();
    let __temp0 = __action25(
        builder,
        __5,
        __6,
        __7,
        __8,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
        __9,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action42<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action9(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
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
fn __action43<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action10(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
        builder,
        __0,
        __temp0,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action44<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> (Outline, Direction)
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action9(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action15(
        builder,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action45<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, Token, usize),
) -> (Outline, Direction)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action10(
        builder,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action15(
        builder,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action46<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, Token, usize),
) -> (Outline, Direction)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action9(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action16(
        builder,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action47<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
) -> (Outline, Direction)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action10(
        builder,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action16(
        builder,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action48<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, Token, usize),
) -> (Outline, Direction)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action9(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        builder,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action49<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
) -> (Outline, Direction)
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action10(
        builder,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        builder,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action50<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
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
    __action36(
        builder,
        __0,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action51<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action52<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action53<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action54<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action55<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action56<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action57<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
        builder,
        __0,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action58<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
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
    __action37(
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
fn __action59<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
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
fn __action60<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
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
fn __action61<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action62<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
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
fn __action63<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action64<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action65<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action66<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
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
    __action38(
        builder,
        __0,
        __temp0,
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
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
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
fn __action68<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
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
fn __action69<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action70<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
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
fn __action71<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action72<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action73<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action74<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
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
    __action39(
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
fn __action75<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
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
fn __action76<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
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
fn __action77<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action78<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
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
fn __action79<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action80<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action81<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action82<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
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
    __action40(
        builder,
        __0,
        __temp0,
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
fn __action83<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action84<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action85<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action86<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action87<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action88<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action89<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action40(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action90<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, Token, usize),
    __5: (usize, String, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
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
    __action41(
        builder,
        __0,
        __temp0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action91<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
    __8: (usize, String, usize),
    __9: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action45(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action92<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
    __8: (usize, String, usize),
    __9: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action46(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action93<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
    __9: (usize, String, usize),
    __10: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action47(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action94<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, Token, usize),
    __5: (usize, Token, usize),
    __6: (usize, String, usize),
    __7: (usize, Token, usize),
    __8: (usize, String, usize),
    __9: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action48(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action95<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
    __9: (usize, String, usize),
    __10: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action96<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
    __9: (usize, String, usize),
    __10: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action97<
>(
    builder: &mut dyn crate::plantuml::parser::tree::Actions,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, Token, usize),
    __7: (usize, String, usize),
    __8: (usize, Token, usize),
    __9: (usize, String, usize),
    __10: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action19(
        builder,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __temp0,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
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
