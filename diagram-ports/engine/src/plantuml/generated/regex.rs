// auto-generated: "lalrpop 0.23.2"
// sha3: e11ae4068040827ed7a2a1cb40684d6fc360d33fd345deae424d6f2038e369ea
// features: 
use crate::plantuml::{regex::{Kind,Member,Upper},parser::{railroad::Token,error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Document {

    use crate::plantuml::{regex::{Kind,Member,Upper},parser::{railroad::Token,error::ParseFailure}};
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    #[allow(unused_extern_crates)]
    extern crate alloc;
    use super::__ToTriple;
    #[allow(dead_code)]
    pub(crate) enum __Symbol<'s>
     {
        Variant0(char),
        Variant1(&'s str),
        Variant2(usize),
        Variant3(Token<'s>),
        Variant4(Member),
        Variant5(Vec<Member>),
        Variant6(()),
        Variant7(Option<()>),
        Variant8((usize, Upper, String)),
    }
    const __ACTION: &[i8] = &[
        // State 0
        16, 17, 0, 0, 4, 5, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 1
        -21, -21, 0, -21, -21, -21, -21, -21, 0, 22, 0, 0, 0, 0, 21, 19, 20, -21,
        // State 2
        16, 17, 0, -11, 4, 5, -11, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 3
        16, 17, 0, 0, 4, 5, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 4
        16, 17, 0, 0, 4, 5, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 5
        28, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0, 0, 0,
        // State 6
        16, 17, 0, 0, 4, 5, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 7
        28, 29, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        28, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        16, 17, 0, -12, 4, 5, -12, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 10
        28, 29, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        -29, -29, 0, -29, -29, -29, -29, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29,
        // State 14
        -4, -4, 0, -4, -4, -4, -4, -4, 0, -4, 0, 0, 0, 0, -4, -4, -4, -4,
        // State 15
        -3, -3, 0, -3, -3, -3, -3, -3, 0, -3, 0, 0, 0, 0, -3, -3, -3, -3,
        // State 16
        -5, -5, 0, -5, -5, -5, -5, -5, 0, -5, 0, 0, 0, 0, -5, -5, -5, -5,
        // State 17
        -22, -22, 0, -22, -22, -22, -22, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22,
        // State 18
        -24, -24, 0, -24, -24, -24, -24, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24,
        // State 19
        -25, -25, 0, -25, -25, -25, -25, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25,
        // State 20
        -23, -23, 0, -23, -23, -23, -23, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23,
        // State 21
        0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 22
        -30, -30, 0, -30, -30, -30, -30, -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30,
        // State 23
        0, 0, 0, 7, 0, 0, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 7, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 25
        -16, -16, 0, 0, 0, 0, 0, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 26
        -18, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 27
        -13, -13, 0, 0, 0, 0, 0, 0, -13, 0, 0, 35, 0, 0, 0, 0, 0, 0,
        // State 28
        -15, -15, 0, 0, 0, 0, 0, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 36, 0, 0, 0, 0,
        // State 30
        -6, -6, 0, -6, -6, -6, -6, -6, 0, -6, 0, 0, 0, 0, -6, -6, -6, -6,
        // State 31
        -7, -7, 0, -7, -7, -7, -7, -7, 0, -7, 0, 0, 0, 0, -7, -7, -7, -7,
        // State 32
        -17, -17, 0, 0, 0, 0, 0, 0, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        -9, -9, 0, -9, -9, -9, -9, -9, 0, -9, 0, 0, 0, 0, -9, -9, -9, -9,
        // State 34
        39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 40, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        -26, -26, 0, -26, -26, -26, -26, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26,
        // State 37
        -8, -8, 0, -8, -8, -8, -8, -8, 0, -8, 0, 0, 0, 0, -8, -8, -8, -8,
        // State 38
        -14, -14, 0, 0, 0, 0, 0, 0, -14, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        -27, -27, 0, -27, -27, -27, -27, -27, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27,
        // State 41
        -28, -28, 0, -28, -28, -28, -28, -28, 0, 0, 0, 0, 0, 0, 0, 0, 0, -28,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 18 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        -21,
        // State 2
        -11,
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
        -12,
        // State 10
        0,
        // State 11
        -31,
        // State 12
        -10,
        // State 13
        -29,
        // State 14
        -4,
        // State 15
        -3,
        // State 16
        -5,
        // State 17
        -22,
        // State 18
        -24,
        // State 19
        -25,
        // State 20
        -23,
        // State 21
        0,
        // State 22
        -30,
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
        -6,
        // State 31
        -7,
        // State 32
        0,
        // State 33
        -9,
        // State 34
        0,
        // State 35
        0,
        // State 36
        -26,
        // State 37
        -8,
        // State 38
        0,
        // State 39
        0,
        // State 40
        -27,
        // State 41
        -28,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 1,
            3 => 11,
            4 => match state {
                3 => 23,
                4 => 24,
                _ => 12,
            },
            5 => match state {
                7 | 10 => 32,
                _ => 25,
            },
            6 => match state {
                8 => 10,
                _ => 7,
            },
            7 => 8,
            9 => match state {
                2 | 9 => 22,
                _ => 13,
            },
            10 => 17,
            11 => match state {
                6 => 9,
                _ => 2,
            },
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""char""###,
        r###""escape""###,
        r###""number""###,
        r###""|""###,
        r###""(""###,
        r###""(?:""###,
        r###"")""###,
        r###""[""###,
        r###""]""###,
        r###""{""###,
        r###""}""###,
        r###""-""###,
        r###""^""###,
        r###"",""###,
        r###""?""###,
        r###""*""###,
        r###""+""###,
        r###"".""###,
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
        's,
        '__1,
        '__2,
    >(
        __states: &[i8],
        _: core::marker::PhantomData<(&'s ())>,
    ) -> alloc::vec::Vec<alloc::string::String>
    where
        '__1: '__2,
    {
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            if __accepts(None, __states, Some(index), core::marker::PhantomData::<(&())>) {
                Some(alloc::string::ToString::to_string(terminal))
            } else {
                None
            }
        }).collect()
    }
    struct __StateMachine<'s, '__1, '__2>
    where '__1: '__2
    {
        builder: &'__2 mut crate::plantuml::regex::Builder<'__1>,
        __phantom: core::marker::PhantomData<(&'s ())>,
    }
    impl<'s, '__1, '__2> __state_machine::ParserDefinition for __StateMachine<'s, '__1, '__2>
    where '__1: '__2
    {
        type Location = usize;
        type Error = ParseFailure;
        type Token = Token<'s>;
        type TokenIndex = usize;
        type Symbol = __Symbol<'s>;
        type Success = usize;
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
                self.builder,
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
        's,
    >(
        __token: &Token<'s>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> Option<usize>
    {
        #[warn(unused_variables)]
        match __token {
            Token::Char(_) if true => Some(0),
            Token::Escape(_) if true => Some(1),
            Token::Number(_) if true => Some(2),
            Token::Pipe if true => Some(3),
            Token::Open if true => Some(4),
            Token::Noncapture if true => Some(5),
            Token::Close if true => Some(6),
            Token::Square if true => Some(7),
            Token::EndSquare if true => Some(8),
            Token::Brace if true => Some(9),
            Token::EndBrace if true => Some(10),
            Token::Minus if true => Some(11),
            Token::Negate if true => Some(12),
            Token::Comma if true => Some(13),
            Token::Question if true => Some(14),
            Token::Star if true => Some(15),
            Token::Plus if true => Some(16),
            Token::Dot if true => Some(17),
            _ => None,
        }
    }
    fn __token_to_symbol<
        's,
    >(
        __token_index: usize,
        __token: Token<'s>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> __Symbol<'s>
    {
        #[allow(clippy::manual_range_patterns)]match __token_index {
            0 => match __token {
                Token::Char(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            1 => match __token {
                Token::Escape(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            2 => match __token {
                Token::Number(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 => __Symbol::Variant3(__token),
            _ => unreachable!(),
        }
    }
    fn __simulate_reduce<
        's,
        '__1,
        '__2,
    >(
        __reduce_index: i8,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> __state_machine::SimulatedReduce<__StateMachine<'s, '__1, '__2>>
    where
        '__1: '__2,
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
                    states_to_pop: 1,
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
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 2,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 3,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 4,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 4,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 5,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 5,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 5,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 6,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 7,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 8,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 9,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 10,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 10,
                }
            }
            27 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 10,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            29 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 11,
                }
            }
            30 => __state_machine::SimulatedReduce::Accept,
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
            's,
            __TOKEN: __ToTriple<'s, >,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::regex::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<usize, __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>
        {
            let __tokens = __tokens0.into_iter();
            let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
            __state_machine::Parser::drive(
                __StateMachine {
                    builder,
                    __phantom: core::marker::PhantomData::<(&())>,
                },
                __tokens,
            )
        }
    }
    fn __accepts<
        's,
        '__1,
        '__2,
    >(
        __error_state: Option<i8>,
        __states: &[i8],
        __opt_integer: Option<usize>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> bool
    where
        '__1: '__2,
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
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> Option<Result<usize,__lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                // Atom = "char" => ActionFn(40);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action40::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 2)
            }
            3 => {
                // Atom = "." => ActionFn(41);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action41::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 2)
            }
            4 => {
                // Atom = "escape" => ActionFn(42);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action42::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 2)
            }
            5 => {
                // Atom = "(", Expression, ")" => ActionFn(43);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action43::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 2)
            }
            6 => {
                // Atom = "(?:", Expression, ")" => ActionFn(44);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action44::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 2)
            }
            7 => {
                // Atom = "[", Negation, Members, "]" => ActionFn(50);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant3(__symbols);
                let __sym2 = __pop_Variant5(__symbols);
                let __sym1 = __pop_Variant6(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action50::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 2)
            }
            8 => {
                // Atom = "[", Members, "]" => ActionFn(51);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant5(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action51::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 2)
            }
            9 => {
                __reduce9(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            11 => {
                // Expression = Expression, "|", Sequence => ActionFn(46);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action46::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 4)
            }
            12 => {
                __reduce12(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            13 => {
                // Member = "char", "-", "char" => ActionFn(47);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action47::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (3, 5)
            }
            14 => {
                // Member = "escape" => ActionFn(25);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action25::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (1, 5)
            }
            15 => {
                // Members = Member => ActionFn(21);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action21::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (1, 6)
            }
            16 => {
                // Members = Members, Member => ActionFn(22);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant4(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action22::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (2, 6)
            }
            17 => {
                __reduce17(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            18 => {
                __reduce18(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            19 => {
                __reduce19(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            20 => {
                __reduce20(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            21 => {
                // Quantified = Atom, Quantifier => ActionFn(48);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant8(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action48::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (2, 9)
            }
            22 => {
                __reduce22(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            23 => {
                __reduce23(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            24 => {
                __reduce24(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            25 => {
                // Quantifier = "{", "number", "}" => ActionFn(11);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action11::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (3, 10)
            }
            26 => {
                // Quantifier = "{", "number", ",", "}" => ActionFn(12);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant3(__symbols);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action12::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (4, 10)
            }
            27 => {
                // Quantifier = "{", "number", ",", "number", "}" => ActionFn(13);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant3(__symbols);
                let __sym3 = __pop_Variant2(__symbols);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action13::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant8(__nt), __end));
                (5, 10)
            }
            28 => {
                __reduce28(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            29 => {
                // Sequence = Sequence, Quantified => ActionFn(49);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action49::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (2, 11)
            }
            30 => {
                // __Document = Document => ActionFn(0);
                let __sym0 = __pop_Variant2(__symbols);
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
    fn __pop_Variant6<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, (usize, Upper, String), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Member, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Option<()>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Token<'s>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Vec<Member>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, char, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, &'s str, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(29);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action29::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(28);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action28::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
    fn __reduce9<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Document = Expression => ActionFn(1);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 3)
    }
    fn __reduce10<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Expression = Sequence => ActionFn(2);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 4)
    }
    fn __reduce12<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Member = "char" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 5)
    }
    fn __reduce17<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Negation = "^" => ActionFn(20);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 7)
    }
    fn __reduce18<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Negation? = Negation => ActionFn(26);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 8)
    }
    fn __reduce19<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Negation? =  => ActionFn(27);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action27::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (0, 8)
    }
    fn __reduce20<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Quantified = Atom => ActionFn(6);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action6::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 9)
    }
    fn __reduce22<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Quantifier = "?" => ActionFn(8);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 10)
    }
    fn __reduce23<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Quantifier = "*" => ActionFn(9);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 10)
    }
    fn __reduce24<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Quantifier = "+" => ActionFn(10);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 10)
    }
    fn __reduce28<
        's,
    >(
        builder: &mut crate::plantuml::regex::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Sequence = Quantified => ActionFn(4);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action4::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 11)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Document::DocumentParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, a, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, b, _): (usize, usize, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Choice(a,b)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, a, _): (usize, usize, usize),
    (_, b, _): (usize, usize, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Sequence(a,b)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, id, _): (usize, usize, usize),
    (_, bounds, _): (usize, (usize, Upper, String), usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.repeat(l..r,id,bounds).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, Token<'s>, usize),
) -> (usize, Upper, String)
{
    (0,Upper::Finite(1),String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, Token<'s>, usize),
) -> (usize, Upper, String)
{
    (0,Upper::Unbounded,String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, Token<'s>, usize),
) -> (usize, Upper, String)
{
    (1,Upper::Unbounded,String::new())
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, _, _): (usize, Token<'s>, usize),
    (_, n, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
) -> Result<(usize, Upper, String),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.bounds(n,Upper::Finite(n)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, _, _): (usize, Token<'s>, usize),
    (_, n, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, _, _): (usize, Token<'s>, usize),
) -> Result<(usize, Upper, String),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.bounds(n,Upper::Unbounded).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, _, _): (usize, Token<'s>, usize),
    (_, n, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, m, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
) -> Result<(usize, Upper, String),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.bounds(n,Upper::Finite(m)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, ch, _): (usize, char, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.text(&ch.to_string()).and_then(|text|builder.node(l..r,Kind::Literal(text))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Dot).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, raw, _): (usize, &'s str, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.text(raw).and_then(|text|builder.node(l..r,Kind::Escape(text))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Group(id,"capture")).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Group(id,"noncapture")).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, neg, _): (usize, Option<()>, usize),
    (_, members, _): (usize, Vec<Member>, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.class(l..r,neg.is_some(),members).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, Token<'s>, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, member, _): (usize, Member, usize),
) -> Result<Vec<Member>,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    { let mut members=Vec::new(); builder.member(&mut members,member).map(|_| members).map_err(|error|lalrpop_util::ParseError::User{error}) }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, mut members, _): (usize, Vec<Member>, usize),
    (_, member, _): (usize, Member, usize),
) -> Result<Vec<Member>,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.member(&mut members,member).map(|_| members).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, ch, _): (usize, char, usize),
) -> Member
{
    Member::Scalar(ch)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, a, _): (usize, char, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, b, _): (usize, char, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<Member,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.range(l..r,a,b).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, raw, _): (usize, &'s str, usize),
) -> Result<Member,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.text(raw).map(Member::Escape).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    (_, __0, _): (usize, (), usize),
) -> Option<()>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<()>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action28<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action29<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action30<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, char, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action31<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action15(
        builder,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action32<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action16(
        builder,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action33<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
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
fn __action34<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action18(
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
fn __action35<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, Option<()>, usize),
    __2: (usize, Vec<Member>, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action36<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
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
fn __action37<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, char, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, char, usize),
    __3: (usize, usize, usize),
) -> Result<Member,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action38<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (usize, Upper, String), usize),
    __2: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action39<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, usize, usize),
    __2: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action5(
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
fn __action40<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, char, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action30(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action41<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action42<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, &'s str, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action32(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action43<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action33(
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
fn __action44<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
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
fn __action45<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, Option<()>, usize),
    __2: (usize, Vec<Member>, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action28(
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
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action46<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action36(
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
fn __action47<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, char, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, char, usize),
) -> Result<Member,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
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
fn __action48<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, (usize, Upper, String), usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action38(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action49<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action39(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action50<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, (), usize),
    __2: (usize, Vec<Member>, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action26(
        builder,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action45(
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
fn __action51<
    's,
>(
    builder: &mut crate::plantuml::regex::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, Vec<Member>, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action27(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action45(
        builder,
        __0,
        __temp0,
        __1,
        __2,
    )
}

#[allow(clippy::type_complexity, dead_code)]
pub trait __ToTriple<'s, >
{
    fn to_triple(self) -> Result<(usize,Token<'s>,usize), __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>;
}

impl<'s, > __ToTriple<'s, > for (usize, Token<'s>, usize)
{
    fn to_triple(self) -> Result<(usize,Token<'s>,usize), __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>> {
        Ok(self)
    }
}
impl<'s, > __ToTriple<'s, > for Result<(usize, Token<'s>, usize), ParseFailure>
{
    fn to_triple(self) -> Result<(usize,Token<'s>,usize), __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>> {
        self.map_err(|error| __lalrpop_util::ParseError::User { error })
    }
}
