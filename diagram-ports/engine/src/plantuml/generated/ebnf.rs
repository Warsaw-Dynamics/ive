// auto-generated: "lalrpop 0.23.2"
// sha3: b818d4d81d89943236a25ec0b02ce61d2c2b526d5a129475ee1c3f7da01a432e
// features: 
use crate::plantuml::{ebnf::Kind, parser::{railroad::Token,error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Document {

    use crate::plantuml::{ebnf::Kind, parser::{railroad::Token,error::ParseFailure}};
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
        Variant0(&'s str),
        Variant1((&'s str, char)),
        Variant2(Token<'s>),
        Variant3(usize),
        Variant4(()),
        Variant5(Option<usize>),
        Variant6(Option<()>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        19, 18, 0, 17, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 3
        19, 18, 0, 0, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 4
        19, 18, 0, 0, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 5
        19, 18, 0, 0, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 6
        19, 18, 0, 0, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 7
        19, 18, 0, 0, 0, 0, 4, 0, 5, 0, 6, 0, 0,
        // State 8
        0, 0, 0, -8, -8, -8, 0, -8, 0, -8, 0, -8, 29,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, -21, -21, -21, 0, -21, 0, -21, 0, -21, 0,
        // State 14
        0, 0, 0, 20, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        0, 0, 0, -10, 8, -10, 0, -10, 0, -10, 0, -10, 0,
        // State 16
        -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, -4, -4, -4, 0, -4, 0, -4, 0, -4, 0,
        // State 18
        0, 0, 0, -3, -3, -3, 0, -3, 0, -3, 0, -3, 0,
        // State 19
        -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 20
        0, 0, 0, 0, 0, 7, 0, 26, 0, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 7, 0, 0, 0, 27, 0, 0, 0,
        // State 22
        0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 9, 0,
        // State 23
        0, 0, 0, -11, 8, -11, 0, -11, 0, -11, 0, -11, 0,
        // State 24
        0, 0, 0, -22, -22, -22, 0, -22, 0, -22, 0, -22, 0,
        // State 25
        0, 0, 0, -5, -5, -5, 0, -5, 0, -5, 0, -5, 0,
        // State 26
        0, 0, 0, -6, -6, -6, 0, -6, 0, -6, 0, -6, 0,
        // State 27
        0, 0, 0, -7, -7, -7, 0, -7, 0, -7, 0, -7, 0,
        // State 28
        0, 0, 0, -14, -14, -14, 0, -14, 0, -14, 0, -14, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 13 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        -9,
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
        -23,
        // State 10
        -19,
        // State 11
        0,
        // State 12
        -20,
        // State 13
        0,
        // State 14
        0,
        // State 15
        0,
        // State 16
        -18,
        // State 17
        0,
        // State 18
        0,
        // State 19
        -17,
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
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => match state {
                7 => 24,
                _ => 13,
            },
            3 => 9,
            4 => match state {
                3 => 20,
                4 => 21,
                5 => 22,
                _ => 14,
            },
            6 => 27,
            8 => match state {
                1 => 12,
                _ => 10,
            },
            9 => 1,
            10 => match state {
                6 => 23,
                _ => 15,
            },
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""word""###,
        r###""text""###,
        r###""=""###,
        r###"";""###,
        r###"",""###,
        r###""|""###,
        r###""(""###,
        r###"")""###,
        r###""[""###,
        r###""]""###,
        r###""{""###,
        r###""}""###,
        r###""-""###,
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
        builder: &'__2 mut crate::plantuml::ebnf::Builder<'__1>,
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
            __token_to_integer(token, core::marker::PhantomData::<(&())>)
        }

        #[inline]
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 13 - 1)
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
            Token::Word(_) if true => Some(0),
            Token::Text(_) if true => Some(1),
            Token::Eq if true => Some(2),
            Token::Semi if true => Some(3),
            Token::Comma if true => Some(4),
            Token::Pipe if true => Some(5),
            Token::Open if true => Some(6),
            Token::Close if true => Some(7),
            Token::Square if true => Some(8),
            Token::EndSquare if true => Some(9),
            Token::Brace if true => Some(10),
            Token::EndBrace if true => Some(11),
            Token::Minus if true => Some(12),
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
                Token::Word(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            1 => match __token {
                Token::Text(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
            2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 => __Symbol::Variant2(__token),
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
                    states_to_pop: 3,
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
                    states_to_pop: 4,
                    nonterminal_produced: 2,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 2,
                }
            }
            8 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 3,
                }
            }
            9 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 4,
                }
            }
            10 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 4,
                }
            }
            11 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 5,
                }
            }
            12 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 0,
                    nonterminal_produced: 5,
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
                    nonterminal_produced: 7,
                }
            }
            16 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 4,
                    nonterminal_produced: 8,
                }
            }
            17 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 8,
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
                    states_to_pop: 1,
                    nonterminal_produced: 10,
                }
            }
            21 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 3,
                    nonterminal_produced: 10,
                }
            }
            22 => __state_machine::SimulatedReduce::Accept,
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
            builder: &mut crate::plantuml::ebnf::Builder<'_>,
            __tokens0: __TOKENS,
        ) -> Result<(), __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>
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
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> Option<Result<(),__lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                // Atom = "word" => ActionFn(29);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action29::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (1, 2)
            }
            3 => {
                // Atom = "text" => ActionFn(30);
                let __sym0 = __pop_Variant1(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = match super::__action30::<>(builder, __sym0) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (1, 2)
            }
            4 => {
                // Atom = "(", Expression, ")" => ActionFn(31);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action31::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 2)
            }
            5 => {
                // Atom = "[", Expression, "]" => ActionFn(32);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action32::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 2)
            }
            6 => {
                // Atom = "{", Expression, "}", Plus => ActionFn(39);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant4(__symbols);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action39::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 2)
            }
            7 => {
                // Atom = "{", Expression, "}" => ActionFn(40);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant3(__symbols);
                let __sym0 = __pop_Variant2(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action40::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 2)
            }
            8 => {
                __reduce8(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            9 => {
                __reduce9(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            10 => {
                // Expression = Expression, "|", Sequence => ActionFn(34);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action34::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 4)
            }
            11 => {
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            12 => {
                __reduce12(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            13 => {
                __reduce13(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            14 => {
                __reduce14(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            15 => {
                __reduce15(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            16 => {
                // Rule = "word", "=", Expression, ";" => ActionFn(37);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant2(__symbols);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action37::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (4, 8)
            }
            17 => {
                // Rule = "word", "=", ";" => ActionFn(38);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant2(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action38::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant4(__nt), __end));
                (3, 8)
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
                // Sequence = Sequence, ",", Atom => ActionFn(36);
                assert!(__symbols.len() >= 3);
                let __sym2 = __pop_Variant3(__symbols);
                let __sym1 = __pop_Variant2(__symbols);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = match super::__action36::<>(builder, __sym0, __sym1, __sym2) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (3, 10)
            }
            22 => {
                // __Document = Document => ActionFn(0);
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
    fn __pop_Variant4<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, (&'s str, char), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Option<()>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Option<usize>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Token<'s>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, &'s str, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __reduce0<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(20);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action20::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(17);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action17::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 1)
    }
    fn __reduce8<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Document = Rules => ActionFn(1);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 3)
    }
    fn __reduce9<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Expression = Sequence => ActionFn(5);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action5::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 4)
    }
    fn __reduce11<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Expression? = Expression => ActionFn(18);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 5)
    }
    fn __reduce12<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Expression? =  => ActionFn(19);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action19::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
    }
    fn __reduce13<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Plus = "-" => ActionFn(14);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action14::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 6)
    }
    fn __reduce14<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Plus? = Plus => ActionFn(15);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action15::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 7)
    }
    fn __reduce15<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Plus? =  => ActionFn(16);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action16::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
    }
    fn __reduce18<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Rules = Rule => ActionFn(2);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 9)
    }
    fn __reduce19<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Rules = Rules, Rule => ActionFn(3);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action3::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 9)
    }
    fn __reduce20<
        's,
    >(
        builder: &mut crate::plantuml::ebnf::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Sequence = Atom => ActionFn(7);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 10)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Document::DocumentParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, (), usize),
    (_, __1, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action4<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, name, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, expr, _): (usize, Option<usize>, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    {
    let id = match expr { Some(id) => Ok(id), None => builder.node(l..r,Kind::Empty) };
    id.and_then(|id| builder.rule(l..r,name,id)).map_err(|error|lalrpop_util::ParseError::User{error})
}
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
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
fn __action7<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> usize
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, a, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, b, _): (usize, usize, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Sequence(a,b)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, name, _): (usize, &'s str, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.text(name).and_then(|text|builder.node(l..r,Kind::Reference(text))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, raw, _): (usize, (&'s str, char), usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.text(raw.0).and_then(|text|builder.node(l..r,Kind::Terminal(text,raw.1))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action11<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Group(id)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action12<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Optional(id)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action13<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, min, _): (usize, Option<()>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.node(l..r,Kind::Repeat(id,usize::from(min.is_some()))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action14<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, Token<'s>, usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action15<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, (), usize),
) -> Option<()>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action16<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<()>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action17<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action18<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    (_, __0, _): (usize, usize, usize),
) -> Option<usize>
{
    Some(__0)
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action19<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Option<usize>
{
    None
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action20<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action21<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action22<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, (&'s str, char), usize),
    __1: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action23<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action11(
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
fn __action24<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action25<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, Option<()>, usize),
    __4: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action13(
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
fn __action26<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action27<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, Option<usize>, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action28<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
    __3: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action20(
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
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action29<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action21(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action30<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, (&'s str, char), usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        builder,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action31<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action23(
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
fn __action32<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action24(
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
fn __action33<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, Option<()>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action25(
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
fn __action34<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action17(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action26(
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
fn __action35<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, Option<usize>, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action17(
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
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action36<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, usize, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action17(
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

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action37<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action18(
        builder,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
        builder,
        __0,
        __1,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action38<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action19(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
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
fn __action39<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
    __3: (usize, (), usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action15(
        builder,
        __3,
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
fn __action40<
    's,
>(
    builder: &mut crate::plantuml::ebnf::Builder<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, usize, usize),
    __2: (usize, Token<'s>, usize),
) -> Result<usize,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action16(
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
