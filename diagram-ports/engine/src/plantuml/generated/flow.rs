// auto-generated: "lalrpop 0.23.2"
// sha3: 5bafb0b3727ebc94464452932e23b6a9a0b6923ee8a19d1f1e9ec9c136d4f69f
// features: 
use crate::plantuml::{flow::Cardinal,parser::{flow::Token,error::ParseFailure}};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Document {

    use crate::plantuml::{flow::Cardinal,parser::{flow::Token,error::ParseFailure}};
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
        Variant1(Token<'s>),
        Variant2(usize),
        Variant3(()),
    }
    const __ACTION: &[i8] = &[
        // State 0
        8, 0, 0, 0, 0,
        // State 1
        8, 0, 0, 0, 0,
        // State 2
        12, 4, 0, 0, 0,
        // State 3
        0, -11, 15, 0, 0,
        // State 4
        0, 4, 0, 0, 0,
        // State 5
        -7, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 3, 10,
        // State 8
        -8, 0, 0, 0, 0,
        // State 9
        -3, 0, 0, 0, 0,
        // State 10
        0, 0, 0, 0, 13,
        // State 11
        0, 0, 0, 5, 16,
        // State 12
        -4, 0, 0, 0, 0,
        // State 13
        0, 17, 0, 0, 0,
        // State 14
        0, -12, 0, 0, 0,
        // State 15
        -5, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, -10,
        // State 17
        0, 0, 0, 0, 19,
        // State 18
        -6, 0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 5 + integer]
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
        -7,
        // State 6
        -13,
        // State 7
        0,
        // State 8
        -8,
        // State 9
        -3,
        // State 10
        0,
        // State 11
        0,
        // State 12
        -4,
        // State 13
        0,
        // State 14
        0,
        // State 15
        -5,
        // State 16
        0,
        // State 17
        0,
        // State 18
        -6,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => match state {
                1 => 8,
                _ => 5,
            },
            3 => 1,
            4 => 6,
            5 => match state {
                4 => 17,
                _ => 10,
            },
            6 => 13,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""word""###,
        r###""quote""###,
        r###""text""###,
        r###""gap""###,
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
        builder: &'__2 mut crate::plantuml::flow::Builder<'__1>,
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
            __action(state, 5 - 1)
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
            Token::Quote if true => Some(1),
            Token::Text(_) if true => Some(2),
            Token::Gap if true => Some(3),
            Token::Newline if true => Some(4),
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
            0 | 2 => match __token {
                Token::Word(__tok0) | Token::Text(__tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            1 | 3 | 4 => __Symbol::Variant1(__token),
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
                    states_to_pop: 4,
                    nonterminal_produced: 2,
                }
            }
            5 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 6,
                    nonterminal_produced: 2,
                }
            }
            6 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 1,
                    nonterminal_produced: 3,
                }
            }
            7 => {
                __state_machine::SimulatedReduce::Reduce {
                    states_to_pop: 2,
                    nonterminal_produced: 3,
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
                    states_to_pop: 3,
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
                    states_to_pop: 1,
                    nonterminal_produced: 6,
                }
            }
            12 => __state_machine::SimulatedReduce::Accept,
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
            builder: &mut crate::plantuml::flow::Builder<'_>,
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
        builder: &mut crate::plantuml::flow::Builder<'_>,
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
                // Command = "word", "newline" => ActionFn(17);
                assert!(__symbols.len() >= 2);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = match super::__action17::<>(builder, __sym0, __sym1) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (2, 2)
            }
            3 => {
                // Command = "word", "gap", Quoted, "newline" => ActionFn(18);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action18::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 2)
            }
            4 => {
                // Command = "word", "gap", "word", "newline" => ActionFn(19);
                assert!(__symbols.len() >= 4);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = match super::__action19::<>(builder, __sym0, __sym1, __sym2, __sym3) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (4, 2)
            }
            5 => {
                // Command = "word", "gap", "word", "gap", Quoted, "newline" => ActionFn(20);
                assert!(__symbols.len() >= 6);
                let __sym5 = __pop_Variant1(__symbols);
                let __sym4 = __pop_Variant0(__symbols);
                let __sym3 = __pop_Variant1(__symbols);
                let __sym2 = __pop_Variant0(__symbols);
                let __sym1 = __pop_Variant1(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = match super::__action20::<>(builder, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (6, 2)
            }
            6 => {
                __reduce6(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            7 => {
                __reduce7(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            8 => {
                __reduce8(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            9 => {
                __reduce9(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            10 => {
                __reduce10(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            11 => {
                __reduce11(builder, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            12 => {
                // __Document = Document => ActionFn(0);
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
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, (), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      's,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>
    ) -> (usize, Token<'s>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
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
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(12);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action12::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(11);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action11::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
    fn __reduce6<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Commands = Command => ActionFn(2);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action2::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 3)
    }
    fn __reduce7<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Commands = Commands, Command => ActionFn(3);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action3::<>(builder, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 3)
    }
    fn __reduce8<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Document = Commands => ActionFn(1);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 4)
    }
    fn __reduce9<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Quoted = "quote", Text, "quote" => ActionFn(8);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action8::<>(builder, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (3, 5)
    }
    fn __reduce10<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Text =  => ActionFn(9);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action9::<>(builder, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (0, 6)
    }
    fn __reduce11<
        's,
    >(
        builder: &mut crate::plantuml::flow::Builder<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // Text = "text" => ActionFn(10);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(builder, __sym0);
        __symbols.push((__start, __Symbol::Variant0(__nt), __end));
        (1, 6)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Document::DocumentParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action2<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, __0, _): (usize, (), usize),
)
{
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action3<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
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
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, id, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.link(l..r,Cardinal::South,id).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action5<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, id, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, label, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.tile(l..r,Cardinal::South,(id,label)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action6<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, dir, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.direction(dir,l..r).and_then(|direction|builder.link(l..r,direction,id)).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action7<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, l, _): (usize, usize, usize),
    (_, dir, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, id, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, label, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, r, _): (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    builder.direction(dir,l..r).and_then(|direction|builder.tile(l..r,direction,(id,label))).map_err(|error|lalrpop_util::ParseError::User{error})
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action8<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, _, _): (usize, Token<'s>, usize),
    (_, label, _): (usize, &'s str, usize),
    (_, _, _): (usize, Token<'s>, usize),
) -> &'s str
{
    label
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action9<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> &'s str
{
    ""
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action10<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    (_, value, _): (usize, &'s str, usize),
) -> &'s str
{
    value
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action11<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action12<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action13<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action12(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action14<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action12(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action15<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action12(
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
fn __action16<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, &'s str, usize),
    __5: (usize, Token<'s>, usize),
    __6: (usize, usize, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action12(
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
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action17<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action11(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action13(
        builder,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action18<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action11(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action14(
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
fn __action19<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action11(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action15(
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
fn __action20<
    's,
>(
    builder: &mut crate::plantuml::flow::Builder<'_>,
    __0: (usize, &'s str, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, &'s str, usize),
    __5: (usize, Token<'s>, usize),
) -> Result<(),__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action11(
        builder,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action16(
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
