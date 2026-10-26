// auto-generated: "lalrpop 0.23.2"
// sha3: fbd093f119046a4767fd98910bebd7bae98c1917ec7eef37a3ce28ccd4313885
// features: 
use crate::plantuml::{chart::{Command,Series,SeriesKind,SeriesColor,LabelsMode,LegendPosition},parser::{chart::Token,error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Chart {

    use crate::plantuml::{chart::{Command,Series,SeriesKind,SeriesColor,LabelsMode,LegendPosition},parser::{chart::Token,error::ParseFailure}};
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
        Variant2(f64),
        Variant3(crate::plantuml::style::Color),
        Variant4(usize),
        Variant5(Vec<String>),
        Variant6(()),
        Variant7(Command),
        Variant8(LabelsMode),
        Variant9(LegendPosition),
        Variant10(Vec<f64>),
        Variant11(SeriesColor),
        Variant12(SeriesKind),
    }
    const __ACTION: &[i8] = &[
        // State 0
        -11, -11, -11, -11, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        5, 6, 13, 14, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, 0, 0, 0, 17, 0, 0, 0, 0,
        // State 3
        0, 0, 0, 0, 0, 20, 21, 22, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, 0, 0, 0, 17, 0, 0, 0, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, -25, 0, 0, -25,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 0, 0, 0, 0,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, -21, 0, 0, 0, 0, 0, 0, 36, 0, -21,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0, 0, 0, 0, 0, -13,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, 0, 0, 0, -23, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, 0, 0, 0, -24, 0, 0, 0, 0,
        // State 14
        -12, -12, -12, -12, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, 0, 0, 0, 0, -26, 0, 0, -26,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -10,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18,
        // State 19
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17,
        // State 22
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 23
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, -7,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 0, 0, 0,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 31, 0, 0, 0, 0, 0, 0,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, -19, 0, 0, 0, 0, 0, 0,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 33, 32, 0, 0, 0, 0, 0, 0,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -3, -3, 0, 0, 0, 0, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0,
        // State 32
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -8,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, -20, 0, 0, 0, 0, 0, 0,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, -22,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4, -4, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -9,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 19 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -11,
        // State 1
        -5,
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
        -27,
        // State 11
        0,
        // State 12
        0,
        // State 13
        0,
        // State 14
        -12,
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
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 27,
            3 => 10,
            4 => 11,
            5 => 1,
            6 => 37,
            7 => 17,
            8 => 25,
            9 => 9,
            10 => 2,
            11 => match state {
                4 => 22,
                5 => 23,
                _ => 15,
            },
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""x-axis""###,
        r###""y-axis""###,
        r###""bar""###,
        r###""line""###,
        r###""legend""###,
        r###""left""###,
        r###""right""###,
        r###""top""###,
        r###""bottom""###,
        r###""labels""###,
        r###""[""###,
        r###""]""###,
        r###"",""###,
        r###""arrow""###,
        r###""text""###,
        r###""number""###,
        r###""color""###,
        r###""unknown""###,
        r###""end""###,
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
        builder: &'__1 mut crate::plantuml::chart::Builder<'__0>,
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
            __action(state, 19 - 1)
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
            Token::XAxis if true => Some(0),
            Token::YAxis if true => Some(1),
            Token::Bar if true => Some(2),
            Token::Line if true => Some(3),
            Token::Legend if true => Some(4),
            Token::Left if true => Some(5),
            Token::Right if true => Some(6),
            Token::Top if true => Some(7),
            Token::Bottom if true => Some(8),
            Token::Labels if true => Some(9),
            Token::Open if true => Some(10),
            Token::Close if true => Some(11),
            Token::Comma if true => Some(12),
            Token::Arrow if true => Some(13),
            Token::Text(_) if true => Some(14),
            Token::Number(_) if true => Some(15),
            Token::Color(_) if true => Some(16),
            Token::Unknown(_) if true => Some(17),
            Token::End if true => Some(18),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 18 => __Symbol::Variant0(__token),
            14 | 17 => match __token {
                Token::Text(__tok0) | Token::Unknown(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            15 => match __token {
                Token::Number(__tok0) if true => __Symbol::Variant2(__tok0),
                _ => unreachable!(),
            },
            16 => match __token {
                Token::Color(__tok0) if true => __Symbol::Variant3(__tok0),
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
                    states_to_pop: 3,
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
                    states_to_pop: 5,
                    nonterminal_produced: 4,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 4,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 5,
                    nonterminal_produced: 4,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 7,
                    nonterminal_produced: 4,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
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
                    states_to_pop: 3,
                    nonterminal_produced: 5,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
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
                    states_to_pop: 1,
                    nonterminal_produced: 7,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 7,
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
                    states_to_pop: 3,
                    nonterminal_produced: 8,
                }
            }
            20 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 9,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
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
                    states_to_pop: 0,
                    nonterminal_produced: 11,
                }
            }
            25 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 11,
                }
            }
            26 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct ChartParser {
        _priv: (),
    }

    impl Default for ChartParser { fn default() -> Self { Self::new() } }
    impl ChartParser {
        pub fn new() -> ChartParser {
            ChartParser {
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            __TOKEN: __ToTriple<>,
            __TOKENS: IntoIterator<Item=__TOKEN>,
        >(
            &self,
            builder: &mut crate::plantuml::chart::Builder<'_>,
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
        builder: &mut crate::plantuml::chart::Builder<'_>,
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
                // Categories = "text" => ActionFn(21);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action21::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (1, 2)
            }
            3 => {
                // Categories = Categories, ",", "text" => ActionFn(22);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant5(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action22::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant5(__nt), __end));
                (3, 2)
            }
            4 => {
                // Chart = Commands => ActionFn(1);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action1::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (1, 3)
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
                // Commands = Commands, Command, "end" => ActionFn(30);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant7(__symbols);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action30::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant6(__nt), __end));
                (3, 5)
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
                // Samples = "number" => ActionFn(23);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action23::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant10(__nt), __end));
                (1, 8)
            }
            19 => {
                // Samples = Samples, ",", "number" => ActionFn(24);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant10(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action24::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant10(__nt), __end));
                (3, 8)
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
                __reduce23(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            24 => {
                __reduce24(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            25 => {
                __reduce25(builder, __lookahead_start, __symbols, core::marker::PhantomData::<()>)
            }
            26 => {
                // __Chart = Chart => ActionFn(0);
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
    fn __pop_Variant6<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Command, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, LabelsMode, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, LegendPosition, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant11<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, SeriesColor, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant12<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, SeriesKind, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
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
    fn __pop_Variant5<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, Vec<f64>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, crate::plantuml::style::Color, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>
    ) -> (usize, f64, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
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
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(26);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action26::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(25);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action25::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 1)
    }
    fn __reduce5<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Command = "x-axis", Title, "[", Categories, "]" => ActionFn(4);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant5(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action4::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (5, 4)
    }
    fn __reduce6<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Command = "y-axis", Title => ActionFn(5);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action5::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 4)
    }
    fn __reduce7<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Command = "y-axis", Title, "number", "arrow", "number" => ActionFn(6);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant2(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action6::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (5, 4)
    }
    fn __reduce8<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Command = SeriesKind, Title, "[", Samples, "]", SeriesColor, Labels => ActionFn(29);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant8(__symbols);
        let __sym5 = __pop_Variant11(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant12(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action29::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (7, 4)
    }
    fn __reduce9<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Command = "legend", Position => ActionFn(8);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant9(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action8::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 4)
    }
    fn __reduce10<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Commands =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 5)
    }
    fn __reduce12<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Labels =  => ActionFn(15);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action15::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 6)
    }
    fn __reduce13<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Labels = "labels" => ActionFn(16);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action16::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 6)
    }
    fn __reduce14<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Position = "left" => ActionFn(17);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 7)
    }
    fn __reduce15<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Position = "right" => ActionFn(18);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 7)
    }
    fn __reduce16<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Position = "top" => ActionFn(19);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action19::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 7)
    }
    fn __reduce17<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Position = "bottom" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 7)
    }
    fn __reduce20<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // SeriesColor =  => ActionFn(13);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action13::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 9)
    }
    fn __reduce21<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // SeriesColor = "color" => ActionFn(14);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action14::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 9)
    }
    fn __reduce22<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // SeriesKind = "bar" => ActionFn(11);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (1, 10)
    }
    fn __reduce23<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // SeriesKind = "line" => ActionFn(12);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (1, 10)
    }
    fn __reduce24<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Title =  => ActionFn(9);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action9::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (0, 11)
    }
    fn __reduce25<
    >(
        builder: &mut crate::plantuml::chart::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<>,usize)>,
        _: core::marker::PhantomData<()>,
    ) -> (usize, usize)
    {
        // Title = "text" => ActionFn(10);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 11)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Chart::ChartParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, (), usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.budget.charge(1).map_err(|error|lalrpop_util::ParseError::User{error:ParseFailure::terminal(error)})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, _commands, _): (usize, (), usize),
    (_, l, _): (usize, usize, usize),
    (_, command, _): (usize, Command, usize),
    (_, _, _): (usize, Token, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.command(command,l..r).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, title, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, categories, _): (usize, Vec<String>, usize),
    (_, _, _): (usize, Token, usize),
) -> Command
{
    Command::Categories(title,categories)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, title, _): (usize, String, usize),
) -> Command
{
    Command::AxisTitle(title)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, title, _): (usize, String, usize),
    (_, min, _): (usize, f64, usize),
    (_, _, _): (usize, Token, usize),
    (_, max, _): (usize, f64, usize),
) -> Command
{
    Command::AxisRange(title,min,max)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, kind, _): (usize, SeriesKind, usize),
    (_, name, _): (usize, String, usize),
    (_, _, _): (usize, Token, usize),
    (_, samples, _): (usize, Vec<f64>, usize),
    (_, _, _): (usize, Token, usize),
    (_, color, _): (usize, SeriesColor, usize),
    (_, labels, _): (usize, LabelsMode, usize),
    (_, r, _): (usize, usize, usize),
) -> Command
{
    Command::Series(Series{span:l..r,name,kind,samples,color,labels})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, _, _): (usize, Token, usize),
    (_, position, _): (usize, LegendPosition, usize),
) -> Command
{
    Command::Legend(position)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> String
{
    String::new()
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> String
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> SeriesKind
{
    SeriesKind::Bar
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> SeriesKind
{
    SeriesKind::Line
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> SeriesColor
{
    SeriesColor::Palette
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, value, _): (usize, crate::plantuml::style::Color, usize),
) -> SeriesColor
{
    SeriesColor::Rgb(value)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> LabelsMode
{
    LabelsMode::Hidden
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> LabelsMode
{
    LabelsMode::Values
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action17<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> LegendPosition
{
    LegendPosition::Left
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> LegendPosition
{
    LegendPosition::Right
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> LegendPosition
{
    LegendPosition::Top
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action20<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, __0, _): (usize, Token, usize),
) -> LegendPosition
{
    LegendPosition::Bottom
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action21<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, value, _): (usize, String, usize),
) -> Result<Vec<String>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.push(Vec::new(),value).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action22<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, values, _): (usize, Vec<String>, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, String, usize),
) -> Result<Vec<String>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.push(values,value).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action23<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, value, _): (usize, f64, usize),
) -> Result<Vec<f64>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.push(Vec::new(),value).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action24<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    (_, values, _): (usize, Vec<f64>, usize),
    (_, _, _): (usize, Token, usize),
    (_, value, _): (usize, f64, usize),
) -> Result<Vec<f64>,__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    builder.push(values,value).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action25<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action26<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action27<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __0: (usize, SeriesKind, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, Vec<f64>, usize),
    __4: (usize, Token, usize),
    __5: (usize, SeriesColor, usize),
    __6: (usize, LabelsMode, usize),
    __7: (usize, usize, usize),
) -> Command
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action26(
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
        __6,
        __7,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action28<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, Command, usize),
    __2: (usize, Token, usize),
    __3: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __1.0.clone();
    let __temp0 = __action26(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
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
fn __action29<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __0: (usize, SeriesKind, usize),
    __1: (usize, String, usize),
    __2: (usize, Token, usize),
    __3: (usize, Vec<f64>, usize),
    __4: (usize, Token, usize),
    __5: (usize, SeriesColor, usize),
    __6: (usize, LabelsMode, usize),
) -> Command
{
    let __start0 = __6.2.clone();
    let __end0 = __6.2.clone();
    let __temp0 = __action25(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
        builder,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action30<
>(
    builder: &mut crate::plantuml::chart::Builder<'_>,
    __0: (usize, (), usize),
    __1: (usize, Command, usize),
    __2: (usize, Token, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action25(
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
