// auto-generated: "lalrpop 0.23.2"
// sha3: 4c729f5bc76c740531ac2a53c2a52cd9946339227bae2264420dc10993942d12
// features: 
use crate::plantuml::parser::{context::Context, definition::DefinitionSyntax, error::ParseFailure, token::Token};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
#[allow(unused_extern_crates)]
extern crate alloc;

#[rustfmt::skip]
#[allow(explicit_outlives_requirements, non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::needless_lifetimes, clippy::type_complexity, clippy::needless_return, clippy::too_many_arguments, clippy::match_single_binding, clippy::clone_on_copy, clippy::unit_arg)]
mod __parse__Definition {

    use crate::plantuml::parser::{context::Context, definition::DefinitionSyntax, error::ParseFailure, token::Token};
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
        Variant0(Token<'s>),
        Variant1(&'s str),
        Variant2(usize),
        Variant3(DefinitionSyntax<'s>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        3, 0, 0, 0,
        // State 1
        0, 0, 0, 0,
        // State 2
        0, 4, 0, 0,
        // State 3
        0, 0, 5, 0,
        // State 4
        0, 6, 0, 0,
        // State 5
        0, 0, 0, 7,
        // State 6
        0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 4 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        -4,
        // State 2
        0,
        // State 3
        0,
        // State 4
        0,
        // State 5
        0,
        // State 6
        -3,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 1,
            _ => 0,
        }
    }
    #[allow(clippy::needless_raw_string_hashes)]
    const __TERMINAL: &[&str] = &[
        r###""!define""###,
        r###""gap""###,
        r###""name""###,
        r###""literal""###,
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
        context: &'__2 mut Context<'__1>,
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
        type Success = DefinitionSyntax<'s>;
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
            __action(state, 4 - 1)
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
                self.context,
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
            Token::Define if true => Some(0),
            Token::Gap if true => Some(1),
            Token::Name(_) if true => Some(2),
            Token::Literal(_) if true => Some(3),
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
            0 | 1 => __Symbol::Variant0(__token),
            2 | 3 => match __token {
                Token::Name(__tok0) | Token::Literal(__tok0) if true => __Symbol::Variant1(__tok0),
                _ => unreachable!(),
            },
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
                    states_to_pop: 5,
                    nonterminal_produced: 2,
                }
            }
            3 => __state_machine::SimulatedReduce::Accept,
            _ => panic!("invalid reduction index {__reduce_index}")
        }
    }
    pub struct DefinitionParser {
        _priv: (),
    }

    impl Default for DefinitionParser { fn default() -> Self { Self::new() } }
    impl DefinitionParser {
        pub fn new() -> DefinitionParser {
            DefinitionParser {
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
            context: &mut Context<'_>,
            __tokens0: __TOKENS,
        ) -> Result<DefinitionSyntax<'s>, __lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>
        {
            let __tokens = __tokens0.into_iter();
            let mut __tokens = __tokens.map(|t| __ToTriple::to_triple(t));
            __state_machine::Parser::drive(
                __StateMachine {
                    context,
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
        context: &mut Context<'_>,
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> Option<Result<DefinitionSyntax<'s>,__lalrpop_util::ParseError<usize, Token<'s>, ParseFailure>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(context, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(context, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                // Definition = "!define", "gap", "name", "gap", "literal" => ActionFn(5);
                assert!(__symbols.len() >= 5);
                let __sym4 = __pop_Variant1(__symbols);
                let __sym3 = __pop_Variant0(__symbols);
                let __sym2 = __pop_Variant1(__symbols);
                let __sym1 = __pop_Variant0(__symbols);
                let __sym0 = __pop_Variant0(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym4.2.clone();
                let __nt = match super::__action5::<>(context, __sym0, __sym1, __sym2, __sym3, __sym4) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                __symbols.push((__start, __Symbol::Variant3(__nt), __end));
                (5, 2)
            }
            3 => {
                // __Definition = Definition => ActionFn(0);
                let __sym0 = __pop_Variant3(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(context, __sym0);
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
    ) -> (usize, DefinitionSyntax<'s>, usize)
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
    ) -> (usize, Token<'s>, usize)
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
        context: &mut Context<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(3);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action3::<>(context, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 0)
    }
    fn __reduce1<
        's,
    >(
        context: &mut Context<'_>,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'s>,usize)>,
        _: core::marker::PhantomData<(&'s ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(2);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action2::<>(context, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
}
#[allow(unused_imports)]
pub use self::__parse__Definition::DefinitionParser;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action0<
    's,
>(
    context: &mut Context<'_>,
    (_, __0, _): (usize, DefinitionSyntax<'s>, usize),
) -> DefinitionSyntax<'s>
{
    __0
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes, clippy::just_underscores_and_digits, clippy::extra_unused_type_parameters)]
fn __action1<
    's,
>(
    context: &mut Context<'_>,
    (_, _, _): (usize, Token<'s>, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, name_start, _): (usize, usize, usize),
    (_, name, _): (usize, &'s str, usize),
    (_, name_end, _): (usize, usize, usize),
    (_, _, _): (usize, Token<'s>, usize),
    (_, literal_start, _): (usize, usize, usize),
    (_, literal, _): (usize, &'s str, usize),
    (_, literal_end, _): (usize, usize, usize),
) -> Result<DefinitionSyntax<'s>,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    context.finish_definition((name_start, name, name_end), (literal_start, literal, literal_end))
        .map_err(|error| lalrpop_util::ParseError::User { error })
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action2<
    's,
>(
    context: &mut Context<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookbehind
}

#[allow(unused_variables)]
#[allow(clippy::needless_lifetimes)]
fn __action3<
    's,
>(
    context: &mut Context<'_>,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    *__lookahead
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action4<
    's,
>(
    context: &mut Context<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, usize, usize),
    __4: (usize, Token<'s>, usize),
    __5: (usize, &'s str, usize),
    __6: (usize, usize, usize),
) -> Result<DefinitionSyntax<'s>,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __start1 = __4.2.clone();
    let __end1 = __5.0.clone();
    let __temp0 = __action3(
        context,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    let __temp1 = __action3(
        context,
        &__start1,
        &__end1,
    );
    let __temp1 = (__start1, __temp1, __end1);
    __action1(
        context,
        __0,
        __1,
        __temp0,
        __2,
        __3,
        __4,
        __temp1,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments, clippy::needless_lifetimes,
    clippy::just_underscores_and_digits, clippy::clone_on_copy, clippy::unit_arg)]
fn __action5<
    's,
>(
    context: &mut Context<'_>,
    __0: (usize, Token<'s>, usize),
    __1: (usize, Token<'s>, usize),
    __2: (usize, &'s str, usize),
    __3: (usize, Token<'s>, usize),
    __4: (usize, &'s str, usize),
) -> Result<DefinitionSyntax<'s>,__lalrpop_util::ParseError<usize,Token<'s>,ParseFailure>>
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __start1 = __4.2.clone();
    let __end1 = __4.2.clone();
    let __temp0 = __action2(
        context,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    let __temp1 = __action2(
        context,
        &__start1,
        &__end1,
    );
    let __temp1 = (__start1, __temp1, __end1);
    __action4(
        context,
        __0,
        __1,
        __2,
        __temp0,
        __3,
        __4,
        __temp1,
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
