// auto-generated: "lalrpop 0.23.2"
// sha3: 7fa214752be8a478353752e3ab72835873615ed10464c55a112259a1caace262
// features: 
use crate::plantuml::{chen::{ChenNode, Name, Participation}, parser::{chen::Token, error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Chen {

    use crate::plantuml::{chen::{ChenNode, Name, Participation}, parser::{chen::Token, error::ParseFailure}};
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
        Variant2(usize),
        Variant3(()),
        Variant4(ChenNode),
        Variant5(Name),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -15, -15, -15, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        20, 21, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        25, 27, 19, 22, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        -5, -5, -5, -5, -5, 0, 0, -5, 0, 0, 0, 0, 0, 0,
        // State 4
        25, 27, 19, 22, 26, 0, 0, 31, 0, 0, 0, 0, 0, 0,
        // State 5
        25, 27, 19, 22, 0, 0, 0, 0, -7, 0, 10, 0, 0, 0,
        // State 6
        25, 27, 19, 22, 0, 0, 0, 0, 0, -7, 10, 0, 0, 0,
        // State 7
        25, 27, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        -5, -5, -5, -5, -5, 0, 0, -5, 0, 0, 0, 0, 0, 0,
        // State 9
        25, 27, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        25, 27, 19, 22, 26, 0, 0, 41, 0, 0, 0, 0, 0, 0,
        // State 11
        25, 27, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        25, 27, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        -22, -22, -22, -22, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 14
        25, 27, 19, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        -16, -16, -16, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 6, 7, 0, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, -27, 0, -27, -27, 0, -27, -27, -27,
        // State 19
        -17, -17, -17, -17, -17, 0, 0, 0, -25, -25, 0, 0, 0, 0,
        // State 20
        -18, -18, -18, -18, -18, 0, 0, 0, -26, -26, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, -24, 0, -24, -24, 0, -24, -24, -24,
        // State 22
        0, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, -19, 0, 0, 0, 0, 0, 0, -19,
        // State 24
        0, 0, 0, 0, 0, 0, -25, 0, -25, -25, 0, -25, -25, -25,
        // State 25
        0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 26
        0, 0, 0, 0, 0, 0, -26, 0, -26, -26, 0, -26, -26, -26,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35,
        // State 28
        -6, -6, -6, -6, -6, 0, 0, -6, 0, 0, 0, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 0, 37,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, -8, -8, 0, 0, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0,
        // State 34
        -21, -21, -21, -21, -21, 0, 0, -21, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, -20, 0, 0, 0, 0, 0, 0, -20,
        // State 36
        -3, -3, -3, -3, -3, 0, 0, -3, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42,
        // State 38
        -12, -12, -12, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 41
        -11, -11, -11, -11, -11, 0, 0, -11, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48,
        // State 44
        -4, -4, -4, -4, -4, 0, 0, -4, 0, 0, 0, 0, 0, 0,
        // State 45
        -13, -13, -13, -13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 46
        -23, -23, -23, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 47
        -14, -14, -14, -14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 0, 0, -9, -9, 0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 14 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -15,
        // State 1
        -10,
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
        -28,
        // State 16
        -16,
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
        -12,
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
        -13,
        // State 46
        0,
        // State 47
        -14,
        // State 48
        0,
        // State 49
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 28,
            3 => match state {
                8 => 10,
                _ => 4,
            },
            4 => match state {
                6 => 33,
                _ => 31,
            },
            5 => 15,
            6 => 8,
            7 => 16,
            8 => 1,
            9 => 2,
            10 => match state {
                2 => 22,
                _ => 29,
            },
            11 => 3,
            12 => 14,
            13 => match state {
                1 => 17,
                5..=6 => 32,
                7 => 35,
                9 => 39,
                11 => 42,
                12 => 43,
                14 => 48,
                _ => 23,
            },
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""entity""###,
        r###""relationship""###,
        r###""as""###,
        r###""word""###,
        r###""quoted""###,
        r###""space""###,
        r###""{""###,
        r###""}""###,
        r###""-""###,
        r###""=""###,
        r###""(""###,
        r###"")""###,
        r###"",""###,
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
        builder: &'__1 mut crate::plantuml::chen::Builder<'__0>,
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
            __action(state, 14 - 1)
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
            Token::Entity(_) if true => Some(0),
            Token::Relationship(_) if true => Some(1),
            Token::As(_) if true => Some(2),
            Token::Word(_) if true => Some(3),
            Token::Quoted(_) if true => Some(4),
            Token::Space(_) if true => Some(5),
            Token::Open if true => Some(6),
            Token::Close if true => Some(7),
            Token::Minus if true => Some(8),
            Token::Equal if true => Some(9),
            Token::ParenOpen if true => Some(10),
            Token::ParenClose if true => Some(11),
            Token::Comma if true => Some(12),
            Token::Newline if true => Some(13),
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
            0 | 1 | 2 | 3 | 4 | 5 => match __token {
                Token::Entity(__tok0) | Token::Relationship(__tok0) | Token::As(__tok0) | Token::Word(__tok0) | Token::Quoted(__tok0) | Token::Space(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 => __Symbol::Variant1(__token),
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
                    states_to_pop: 4,
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
                    states_to_pop: 0,
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
                    states_to_pop: 6,
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
                    states_to_pop: 3,
                    nonterminal_produced: 6,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 7,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 7,
                }
            }
            13 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
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
                    nonterminal_produced: 10,
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
                    states_to_pop: 4,
                    nonterminal_produced: 11,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 12,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 12,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            24 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            26 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            27 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct ChenParser {
        _priv: (),
    }

    impl Default for ChenParser { fn default() -> Self { Self::new() } }
    impl ChenParser {
        pub fn new() -> ChenParser {
            ChenParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::chen::Builder<'_>,
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
        builder: &mut crate::plantuml::chen::Builder<'_>,
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
                // Attribute = Name, "newline" => ActionFn(37);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action37::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (2, 2)
            }
            3 => {
                // Attribute = CompositeOpen, Attributes, "}", "newline" => ActionFn(13);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action13::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 2)
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
                // Cardinality = Word => ActionFn(38);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action38::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant0(__nt), __end));
                (1, 4)
            }
            8 => {
                // Cardinality = "(", Word, ",", Space, Word, ")" => ActionFn(39);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action39::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant0(__nt), __end));
                (6, 4)
            }
            9 => {
                // Chen = Items => ActionFn(1);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action1::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (1, 5)
            }
            10 => {
                // CompositeOpen = Name, "{", "newline" => ActionFn(40);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action40::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 6)
            }
            11 => {
                // Item = OwnerOpen, Attributes, "}", "newline" => ActionFn(4);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action4::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 7)
            }
            12 => {
                // Item = Word, "-", Cardinality, "-", Word, "newline" => ActionFn(41);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action41::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (6, 7)
            }
            13 => {
                // Item = Word, "=", Cardinality, "=", Word, "newline" => ActionFn(42);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action42::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (6, 7)
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
                // Name = Word => ActionFn(43);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action43::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (1, 10)
            }
            19 => {
                // Name = "quoted", "as", Word => ActionFn(44);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action44::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 10)
            }
            20 => {
                // OwnerOpen = Kind, Name, "{", "newline" => ActionFn(45);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant5(__symbols);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action45::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 11)
            }
            21 => {
                __reduce21(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
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
                // __Chen = Chen => ActionFn(0);
                let __sym0 = __pop_Variant3(__symbols);
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
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, ChenNode, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Name, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
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
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(27);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action27::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(26);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action26::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
    fn __reduce4<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Attributes =  => ActionFn(10);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action10::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 3)
    }
    fn __reduce5<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Attributes = Attributes, Attribute => ActionFn(11);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action11::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 3)
    }
    fn __reduce6<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Cardinality =  => ActionFn(21);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action21::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (0, 4)
    }
    fn __reduce14<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Items =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 8)
    }
    fn __reduce15<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Items = Items, Item => ActionFn(3);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action3::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 8)
    }
    fn __reduce16<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Kind = "entity" => ActionFn(8);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 9)
    }
    fn __reduce17<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Kind = "relationship" => ActionFn(9);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 9)
    }
    fn __reduce21<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Space =  => ActionFn(24);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action24::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (0, 12)
    }
    fn __reduce22<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Space = "space" => ActionFn(25);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action25::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 12)
    }
    fn __reduce23<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Word = "word" => ActionFn(17);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 13)
    }
    fn __reduce24<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Word = "entity" => ActionFn(18);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 13)
    }
    fn __reduce25<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Word = "relationship" => ActionFn(19);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action19::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 13)
    }
    fn __reduce26<
    >(
        builder: &mut crate::plantuml::chen::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Word = "as" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 13)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Chen::ChenParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.finish().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, _open, _): (usize, (), usize),
    (_, _attrs, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, left, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, cardinality, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, right, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.associate((left, right), (cardinality, Participation::Normal), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, left, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, cardinality, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, right, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.associate((left, right), (cardinality, Participation::Bold), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, kind, _): (usize, ChenNode, usize),
    (_, name, _): (usize, Name, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(kind, name, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, String, usize),
) -> ChenNode
{
    ChenNode::Entity
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, String, usize),
) -> ChenNode
{
    ChenNode::Relationship
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.attribute(name, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, _open, _): (usize, (), usize),
    (_, _attrs, _): (usize, (), usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, Name, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.composite(name, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, name, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.single(name, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, display, _): (usize, String, usize),
    (_, _, _): (usize, String, usize),
    (_, alias, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.named(alias, display, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
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
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.atom(value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, left, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, space, _): (usize, String, usize),
    (_, right, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.cardinality((left, space, right), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action26<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
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
    builder: &mut crate::plantuml::chen::Builder<'_>,
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
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
    __2: (usize, usize, usize),
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
    __action12(
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
fn __action29<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
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
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, usize, usize),
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
    __action23(
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
fn __action31<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action32<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, usize, usize),
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
    __action5(
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
fn __action33<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
    __6: (usize, usize, usize),
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
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action34<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action27(
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
fn __action35<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, String, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action27(
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
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action36<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, ChenNode, usize),
    __1: (usize, Name, usize),
    __2: (usize, Token, usize),
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
    __action7(
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
fn __action37<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
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
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action38<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
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
fn __action39<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<String,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
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
        __5,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action40<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, Name, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
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
fn __action41<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action32(
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
fn __action42<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, Token, usize),
    __4: (usize, String, usize),
    __5: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action26(
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
        __3,
        __4,
        __5,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action43<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action44<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, String, usize),
    __2: (usize, String, usize),
) -> Result<Name,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action26(
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
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action45<
>(
    builder: &mut crate::plantuml::chen::Builder<'_>,
    __0: (usize, ChenNode, usize),
    __1: (usize, Name, usize),
    __2: (usize, Token, usize),
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
    __action36(
        builder,
        __0,
        __1,
        __2,
        __3,
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
