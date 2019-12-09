use heck::CamelCase;
use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, ExprBlock, Ident, ItemEnum, ItemFn, Stmt, Token, Type,
};

use crate::fsm::events::Events;
use crate::fsm::{events::Event, states::State};

#[derive(Debug, PartialEq)]
pub(crate) struct TransitionPair {
    pub from: Ident,
    pub to: Ident,
}

impl Parse for TransitionPair {
    /// example transition pair:
    ///
    /// ```text
    /// S1 => S2
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // `S1 => S2`
        //  ^^
        let from = Ident::parse(&input)?;
        // `S1 => S2`
        //     ^^
        let _: Token![=>] = input.parse()?;

        // `S1 => S2`
        //        ^^
        let to = Ident::parse(&input)?;

        Ok(TransitionPair { from, to })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Transition {
    pub event_name: Ident,
    pub pairs: BTreeMap<Ident, BTreeSet<Ident>>,
}

impl Parse for Transition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // EVENT1 [ ... ]
        // ^^^^^^
        let event_name: Ident = input.parse()?;

        // EVENT1 [ ... ]
        //          ^^^
        let block_transition;
        bracketed!(block_transition in input);

        let mut transition_pairs: BTreeMap<Ident, BTreeSet<Ident>> = BTreeMap::new();

        // EVENT1 [ S1 => S2, S1 => S3, ]
        //          ^^^^^^^^^^^^^^^^^^^
        let punctuated_block_transition: Punctuated<TransitionPair, Token![,]> =
            block_transition.parse_terminated(TransitionPair::parse)?;

        for pair in punctuated_block_transition {
            if let Some(v) = transition_pairs.get_mut(&pair.from) {
                v.insert(pair.to);
            } else {
                let mut v = BTreeSet::new();
                v.insert(pair.to);
                transition_pairs.insert(pair.from, v);
            }
        }

        Ok(Transition {
            event_name,
            pairs: transition_pairs,
        })
    }
}

struct AfterExitCase {
    pub from: Ident,
    pub to: Ident,
}

impl ToTokens for AfterExitCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let to = &self.to;
        tokens.extend(quote! {
            State::#to(state) => {
                self.current_state = State::#to(state);
                state.entry();
                Ok(true)
            }
        })
    }
}

struct StateCase {
    pub from: Ident,
    pub tos: BTreeSet<Ident>,
}

impl ToTokens for StateCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = &self.from;

        let after_exit_cases: Vec<_> = self
            .tos
            .iter()
            .map(|v| AfterExitCase {
                from: from.clone(),
                to: v.clone(),
            })
            .collect();

        tokens.extend(quote! {
            State::#from(state) => {
                match state.exit() {
                    Ok(r) =>  {
                        match r {
                            #( #after_exit_cases )*
                            _ => {
                                panic!("cant't go to state from current state")
                            }
                        }
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            }
        })
    }
}

struct EventCase {
    pub event_name: Ident,
    pub pairs: BTreeMap<Ident, BTreeSet<Ident>>,
}

impl ToTokens for EventCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event_name = &self.event_name;

        let state_cases: Vec<_> = self
            .pairs
            .iter()
            .map(|v| StateCase {
                from: v.0.clone(),
                tos: v.1.clone(),
            })
            .collect();

        tokens.extend(quote! {
            Event::#event_name(event) => {
                if let Err(err) = event.on() {
                    return Err(err);
                }
                match self.current_state {
                    #( #state_cases )*
                }
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Transitions(pub Vec<Transition>);

impl Parse for Transitions {
    /// example transitions tokens:
    ///
    /// ```text
    /// Transitions {
    ///     EVENT1 [
    ///         S1 => S2,
    ///         S1 => S3,
    ///     ],
    ///
    ///     EVENT2 [
    ///         S4 => S5,
    ///     ],
    /// }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// Transitions { ... }
        /// -----------
        let magic = Ident::parse(input)?;

        if magic != "Transitions" {
            return Err(input.error("expected Transitions { ... }"));
        }

        let content;
        braced!(content in input);

        let mut transitions: Vec<Transition> = Vec::new();

        let transitions: Punctuated<Transition, Token![,]> =
            content.parse_terminated(Transition::parse)?;
        Ok(Transitions(transitions.into_iter().collect()))
    }
}

impl Transitions {
    pub fn to_event_fn_tokens(&self) -> TokenStream {
        let event_cases: Vec<_> = self
            .0
            .iter()
            .map(|v| EventCase {
                event_name: v.event_name.clone(),
                pairs: v.pairs.clone(),
            })
            .collect();

        quote! {
            fn event(&mut self, event: Event) -> Result<bool, &'static str> {
                match event {
                    #( #event_cases )*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    //    #[test]
    //    fn test_transition_parse_and_to_tokens() {
    //        let transition: Transition = syn::parse2(quote! {
    //            EVENT1 [
    //               S1 => S2,
    //               S1 => S3,
    //            ]
    //        })
    //        .unwrap();
    //
    //        let left = quote! {
    //            mod event1 {
    //                pub enum AfterExitS1 {
    //                    S2,
    //                    S3,
    //                }
    //                pub trait Callback {
    //                    fn on_event1(&self, data: (&str)) -> Result<(), &'static str>;
    //                    fn exit_s1(&self, data: (&str)) -> Result<AfterExitS1, &'static str>;
    //                    fn entry_s2_from_s1(&self, data: (&str));
    //                    fn entry_s3_from_s1(&self, data: (&str));
    //                }
    //            }
    //        };
    //
    //        let mut right = transition.to_def_tokens();
    //
    //        assert_eq!(format!("{}", left), format!("{}", right))
    //    }

    //    #[test]
    //    fn test_transitions_parse_and_to_tokens() {
    //        let transitions: Transitions = syn::parse2(quote! {
    //            Transitions {
    //                EVENT1 [
    //                   S1 => S2,
    //                   S1 => S3,
    //                ],
    //                EVENT2 [
    //                   S2 => S4,
    //                ]
    //            }
    //        })
    //        .unwrap();
    //
    //        let left = quote! {
    //            mod event1 {
    //                pub enum AfterExitS1 {
    //                    S2,
    //                    S3,
    //                }
    //                pub trait Callback {
    //                    fn on_event1(&self, data: (&str)) -> Result<(), &'static str>;
    //                    fn exit_s1(&self, data: (&str)) -> Result<AfterExitS1, &'static str>;
    //                    fn entry_s2_from_s1(&self, data: (&str));
    //                    fn entry_s3_from_s1(&self, data: (&str));
    //                }
    //            }
    //            mod event2 {
    //                pub enum AfterExitS2 {
    //                    S4,
    //                }
    //                pub trait Callback {
    //                    fn on_event2(&self, data: (&str)) -> Result<(), &'static str>;
    //                    fn exit_s2(&self, data: (&str)) -> Result<AfterExitS2, &'static str>;
    //                    fn entry_s4_from_s2(&self, data: (&str));
    //                }
    //            }
    //        };
    //
    //        let mut right = transitions.to_def_tokens();
    //
    //        assert_eq!(format!("{}", left), format!("{}", right))
    //    }
}
