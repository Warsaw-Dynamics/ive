// auto-generated: "lalrpop 0.23.2"
// sha3: 9a9620079af766463a3d4c0cc46809a730c6abbd4f8946b5977aaa3b3111fb4f
// features: 
use crate::plantuml::{salt::{Border, End, LeafKind, Salt}, parser::{salt::Token, error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Salt {

    use crate::plantuml::{salt::{Border, End, LeafKind, Salt}, parser::{salt::Token, error::ParseFailure}};
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
        Variant1(String),
        Variant2(usize),
        Variant3(Border),
        Variant4(()),
        Variant5(End),
        Variant6(Salt),
        Variant7((usize, End)),
    }
    const __ACTION: &[i8] = &[
        // State 0
        11, 13, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 18,
        // State 2
        -9, -9, -9, -9, -9, 0, -9, -9, -9, 0, -9, 0, 0, 0, -9, 0, 0,
        // State 3
        -8, -8, -8, 7, -8, 0, -8, -8, -8, 0, -8, 0, 0, 0, 6, 0, 0,
        // State 4
        11, 13, 12, 0, 25, 0, 26, 24, 23, 0, 22, 0, 0, 0, 0, 0, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 18,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 18,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 18,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -3, -3,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5, -5,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4,
        // State 13
        -12, -12, -12, -12, -12, 0, -12, -12, -12, 0, -12, 0, 0, 0, -12, 0, 0,
        // State 14
        -24, -24, -24, -24, -24, 0, -24, -24, -24, 0, -24, 0, 0, 0, -24, 0, 0,
        // State 15
        -13, -13, -13, -13, -13, 0, -13, -13, -13, 0, -13, 0, 0, 0, -13, 0, 0,
        // State 16
        -11, -11, -11, -11, -11, 0, -11, -11, -11, 0, -11, 0, 0, 0, -11, 0, 0,
        // State 17
        -25, -25, -25, -25, -25, 0, -25, -25, -25, 0, -25, 0, 0, 0, -25, 0, 0,
        // State 18
        -10, -10, -10, -10, -10, 0, -10, -10, -10, 0, -10, 0, 0, 0, -10, 0, 0,
        // State 19
        -28, -28, -28, -28, -28, 0, -28, -28, -28, 0, -28, 0, 0, 0, -28, 0, 0,
        // State 20
        -7, -7, -7, -7, -7, 0, -7, -7, -7, 0, -7, 0, 0, 0, -7, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 31, 32, 0, 0, 0,
        // State 22
        0, 0, 0, 0, 36, 0, 0, 0, 0, 34, 0, 0, 33, 35, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14, -14,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, -15,
        // State 26
        -6, -6, -6, -6, -6, 0, -6, -6, -6, 0, -6, 0, 0, 0, -6, 0, 0,
        // State 27
        -27, -27, -27, -27, -27, 0, -27, -27, -27, 0, -27, 0, 0, 0, -27, 0, 0,
        // State 28
        -29, -29, -29, -29, -29, 0, -29, -29, -29, 0, -29, 0, 0, 0, -29, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, 0, 0,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 0, 0, 0, 0, 0, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 43, 0, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 44, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22, -22,
        // State 38
        0, 0, 0, 0, 0, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, -19,
        // State 42
        0, 0, 0, 0, 0, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, -17,
        // State 44
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, -16,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, -21,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, -18,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, -20,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 17 + integer]
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
        -30,
        // State 9
        -26,
        // State 10
        0,
        // State 11
        0,
        // State 12
        0,
        // State 13
        -12,
        // State 14
        0,
        // State 15
        -13,
        // State 16
        -11,
        // State 17
        -25,
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
        -27,
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
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 1,
            3 => 18,
            4 => 4,
            5 => 3,
            6 => 13,
            7 => match state {
                5 => 26,
                6 => 27,
                7 => 28,
                _ => 14,
            },
            8 => 7,
            9 => 2,
            10 => 15,
            11 => 8,
            12 => match state {
                4 => 19,
                _ => 9,
            },
            13 => 20,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""{""###,
        r###""{+""###,
        r###""{#""###,
        r###""}""###,
        r###""text""###,
        r###""caption""###,
        r###""tree marker""###,
        r###""quote""###,
        r###""[""###,
        r###""]""###,
        r###""(""###,
        r###"")""###,
        r###""X""###,
        r###""space""###,
        r###""*""###,
        r###""column""###,
        r###""row""###,
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
        builder: &'__1 mut crate::plantuml::salt::Builder<'__0>,
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
        type Success = Salt;
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
            __action(state, 17 - 1)
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
            Token::Open if true => Some(0),
            Token::Outer if true => Some(1),
            Token::Grid if true => Some(2),
            Token::Close if true => Some(3),
            Token::Text(_) if true => Some(4),
            Token::Caption(_) if true => Some(5),
            Token::Tree(_) if true => Some(6),
            Token::Quote if true => Some(7),
            Token::BracketOpen if true => Some(8),
            Token::BracketClose if true => Some(9),
            Token::ParenOpen if true => Some(10),
            Token::ParenClose if true => Some(11),
            Token::X if true => Some(12),
            Token::Space if true => Some(13),
            Token::Star if true => Some(14),
            Token::Column if true => Some(15),
            Token::Row if true => Some(16),
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
            0 | 1 | 2 | 3 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => __Symbol::Variant0(__token),
            4 | 5 | 6 => match __token {
                Token::Text(__tok0) | Token::Caption(__tok0) | Token::Tree(__tok0) if true => __Symbol::Variant1(__tok0),
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
                    states_to_pop: 2,
                    nonterminal_produced: 3,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 3,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 4,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
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
                    states_to_pop: 1,
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
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            14 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 8,
                }
            }
            15 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 8,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 8,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            18 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 8,
                }
            }
            19 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 8,
                }
            }
            22 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            23 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 9,
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
                    states_to_pop: 1,
                    nonterminal_produced: 11,
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
                    states_to_pop: 1,
                    nonterminal_produced: 13,
                }
            }
            28 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 13,
                }
            }
            29 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct SaltParser {
        _priv: (),
    }

    impl Default for SaltParser { fn default() -> Self { Self::new() } }
    impl SaltParser {
        pub fn new() -> SaltParser {
            SaltParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::salt::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<Salt, __lalrpop_util::ParseError<usize, Token, ParseFailure>>
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
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> Option<Result<Salt,__lalrpop_util::ParseError<usize, Token, ParseFailure>>>
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
                // Cell = "*", End => ActionFn(43);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant5(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action43::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (2, 3)
            }
            6 => {
                // Cell = CellOpen, Widget => ActionFn(10);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action10::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (2, 3)
            }
            7 => {
                // CellOpen =  => ActionFn(11);
                let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
                let __end = __start.clone();
                let __nt = match super::__action11::<>(builder, &__start, &__end) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (0, 4)
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
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            12 => {
                __reduce12(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            13 => {
                // Leaf = "text" => ActionFn(44);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action44::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 8)
            }
            14 => {
                // Leaf = "tree marker" => ActionFn(45);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action45::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (1, 8)
            }
            15 => {
                // Leaf = "quote", "text", "quote" => ActionFn(46);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action46::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 8)
            }
            16 => {
                // Leaf = "[", "text", "]" => ActionFn(47);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action47::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 8)
            }
            17 => {
                // Leaf = "[", "X", "]", "caption" => ActionFn(48);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action48::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 8)
            }
            18 => {
                // Leaf = "[", "]", "caption" => ActionFn(49);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action49::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 8)
            }
            19 => {
                // Leaf = "[", "space", "]", "caption" => ActionFn(50);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action50::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 8)
            }
            20 => {
                // Leaf = "(", "X", ")", "caption" => ActionFn(51);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action51::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 8)
            }
            21 => {
                // Leaf = "(", ")", "caption" => ActionFn(52);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action52::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (3, 8)
            }
            22 => {
                // Leaf = "(", "space", ")", "caption" => ActionFn(53);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action53::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant2(__nt), __end));
                (4, 8)
            }
            23 => {
                // Open = Border, End => ActionFn(54);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant5(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action54::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (2, 9)
            }
            24 => {
                __reduce24(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            25 => {
                // Salt = Table => ActionFn(1);
                let __sym0 = __pop_Variant7(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action1::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (1, 11)
            }
            26 => {
                // Table = Open, Cells, "}", End => ActionFn(55);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant5(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant4(__symbols);
                let __sym0 = __pop_Variant4(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action55::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant7(__nt), __end));
                (4, 12)
            }
            27 => {
                __reduce27(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            28 => {
                __reduce28(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            29 => {
                // __Salt = Salt => ActionFn(0);
                let __sym0 = __pop_Variant6(__symbols);
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
    fn __pop_Variant4<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (usize, End), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Border, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, End, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Salt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
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
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
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
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(28);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action28::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
    fn __reduce2<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Border = "{" => ActionFn(4);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action4::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 2)
    }
    fn __reduce3<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Border = "{+" => ActionFn(5);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action5::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 2)
    }
    fn __reduce4<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Border = "{#" => ActionFn(6);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action6::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 2)
    }
    fn __reduce8<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Cells =  => ActionFn(7);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action7::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 5)
    }
    fn __reduce9<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Cells = Cells, Cell => ActionFn(8);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action8::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 5)
    }
    fn __reduce10<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Column = "column" => ActionFn(26);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 6)
    }
    fn __reduce11<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // End = Column => ActionFn(24);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action24::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 7)
    }
    fn __reduce12<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // End = Row => ActionFn(25);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action25::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 7)
    }
    fn __reduce24<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Row = "row" => ActionFn(27);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action27::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 10)
    }
    fn __reduce27<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Widget = Table => ActionFn(12);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 13)
    }
    fn __reduce28<
    >(
        builder: &mut crate::plantuml::salt::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Widget = Leaf, End => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 13)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Salt::SaltParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Salt, usize),
) -> Salt
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, table, _): (usize, (usize, End), usize),
) -> Result<Salt,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.finish(table.0).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, _open, _): (usize, (), usize),
    (_, _cells, _): (usize, (), usize),
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, End, usize),
    (_, finish, _): (usize, usize, usize),
) -> Result<(usize, End),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.close(end, start..finish).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, border, _): (usize, Border, usize),
    (_, _end, _): (usize, End, usize),
    (_, finish, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.open(border, start..finish).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Border
{
    Border::None
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Border
{
    Border::Outer
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> Border
{
    Border::Grid
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, End, usize),
    (_, finish, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.span(end, start..finish).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, id, _): (usize, usize, usize),
    (_, widget, _): (usize, (usize, End), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.cell(id, widget).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.start_cell().map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, (usize, End), usize),
) -> (usize, End)
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, widget, _): (usize, usize, usize),
    (_, end, _): (usize, End, usize),
) -> (usize, End)
{
    (widget, end)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::Text, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.tree(value, start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::Field, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::Button, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::CheckboxOn, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::CheckboxOff, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::CheckboxOff, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::RadioOn, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::RadioOff, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, start, _): (usize, usize, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
    (_, end, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.leaf((LeafKind::RadioOff, value), start..end).map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, End, usize),
) -> End
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action25<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, End, usize),
) -> End
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action26<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> End
{
    End::Column
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action27<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> End
{
    End::Row
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action28<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action29<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
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
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, End, usize),
    __2: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
fn __action31<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action32<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action33<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
        __2,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action34<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action35<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
        __4,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action36<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action37<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action20(
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
fn __action38<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
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
fn __action39<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
fn __action40<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action29(
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
fn __action41<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Border, usize),
    __1: (usize, End, usize),
    __2: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action42<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, End, usize),
    __4: (usize, usize, usize),
) -> Result<(usize, End),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action29(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
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
fn __action43<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, End, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action30(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action44<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action45<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action46<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action47<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action48<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action49<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
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
fn __action50<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
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
        __3,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action51<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
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
        __2,
        __3,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action52<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
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
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action53<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Token, usize),
    __1: (usize, Token, usize),
    __2: (usize, Token, usize),
    __3: (usize, String, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action28(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action54<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, Border, usize),
    __1: (usize, End, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action55<
>(
    builder: &mut crate::plantuml::salt::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, (), usize),
    __2: (usize, Token, usize),
    __3: (usize, End, usize),
) -> Result<(usize, End),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action28(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action42(
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
