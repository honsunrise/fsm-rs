use heck::CamelCase;
use heck::SnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, ExprBlock, Ident, Stmt, Token,
};

use crate::fsm::{event::Event, state::State};

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
        // `S1 => S2 }`
        //  ^^
        let from = Ident::parse(&input)?;
        // `S1 => S2 }`
        //     ^^
        let _: Token![=>] = input.parse()?;

        // `S1 => S2 }`
        //        ^^
        let to = Ident::parse(&input)?;

        Ok(TransitionPair { from, to })
    }
}

struct AfterExitEnums(Ident, BTreeSet<Ident>);

impl ToTokens for AfterExitEnums {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = &self.0;
        let to_set: Vec<Ident> = self.1.iter().map(|v| v.clone()).collect();

        let after_exit_name: Ident = format_ident!("AfterExit{}", from.to_string().to_camel_case());

        tokens.extend(quote! {
            pub enum #after_exit_name {
               #(#to_set,)*
            }
        });
    }
}

struct Callbacks(Ident, BTreeSet<Ident>);

impl ToTokens for Callbacks {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = &self.0;
        let entry_fn_names: Vec<_> = self
            .1
            .iter()
            .map(|v| {
                format_ident!(
                    "entry_{}_from_{}",
                    v.to_string().to_snake_case(),
                    from.to_string().to_snake_case()
                )
            })
            .collect();

        let exit_after_enum_name = format_ident!("AfterExit{}", from.to_string().to_camel_case());
        let exit_fn_name = format_ident!("exit_{}", from.to_string().to_snake_case());

        tokens.extend(quote! {
            fn #exit_fn_name(&self, data: (&str)) -> Result<#exit_after_enum_name, &'static str>;

            #(fn #entry_fn_names(&self, data: (&str));)*
        });
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Transition {
    pub event_name: Ident,
    pub pairs: BTreeMap<Ident, BTreeSet<Ident>>,
    pub block: ExprBlock,
}

impl Parse for Transition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // EVENT1 [ ... ] { ... }
        // ^^^^^^
        let event_name: Ident = input.parse()?;

        // EVENT1 [ ... ] { ... }
        //          ^^^
        let block_transition;
        bracketed!(block_transition in input);

        let mut transition_pairs: BTreeMap<Ident, BTreeSet<Ident>> = BTreeMap::new();

        // EVENT1 [ S1 => S2, S1 => S3, ] { ... }
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
        // EVENT1 [ ... ] { ... }
        //                  ^^^
        let block = ExprBlock::parse(input)?;

        Ok(Transition {
            event_name,
            pairs: transition_pairs,
            block,
        })
    }
}

impl ToTokens for Transition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pairs = &self.pairs;
        let block = &self.block;
        let event_name = Ident::new(
            &self.event_name.to_string().to_snake_case(),
            self.event_name.span(),
        );

        let on_event_name = format_ident!("on_{}", event_name.to_string().to_snake_case());
        let after_enums: Vec<_> = pairs
            .iter()
            .map(|v| AfterExitEnums(v.0.clone(), v.1.iter().cloned().collect()))
            .collect();
        let callbacks: Vec<_> = pairs
            .iter()
            .map(|v| Callbacks(v.0.clone(), v.1.iter().cloned().collect()))
            .collect();

        tokens.extend(quote! {
            mod #event_name {

                #( #after_enums )*

                pub trait Callback {
                    fn #on_event_name(&self, data: (&str)) -> Result<(), &'static str>;
                    #( #callbacks )*
                }
            }
        });
    }
}

struct AfterExitCase {
    pub from: Ident,
    pub to: Ident,
}

impl ToTokens for AfterExitCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = &self.from;
        let to = &self.to;
        let after_exit_name = format_ident!("AfterExit{}", from.to_string().to_camel_case());
        let entry_fn_name = format_ident!(
            "entry_{}_from_{}",
            to.to_string().to_snake_case(),
            from.to_string().to_snake_case()
        );

        tokens.extend(quote! {
            #after_exit_name::#to => {
                self.current_state = State::#to;
                self.#entry_fn_name();
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
        let exit_fn_name = format_ident!("exit_{}", from.to_string().to_snake_case());

        let after_exit_cases: Vec<_> = self
            .tos
            .iter()
            .map(|v| AfterExitCase {
                from: from.clone(),
                to: v.clone(),
            })
            .collect();

        tokens.extend(quote! {
            State::#from => {
                match self.#exit_fn_name() {
                    Ok(r) =>  {
                        match r {
                            #( #after_exit_cases )*
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
    pub block: ExprBlock,
}

impl ToTokens for EventCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event_name = &self.event_name;
        let on_event_name = format_ident!("on_{}", event_name.to_string().to_snake_case());

        let state_cases: Vec<_> = self
            .pairs
            .iter()
            .map(|v| StateCase {
                from: v.0.clone(),
                tos: v.1.clone(),
            })
            .collect();

        tokens.extend(quote! {
            Event::#event_name => {
                if let Err(err) = self.#on_event_name() {
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
    /// EVENT1 [
    ///    S1 => S2,
    ///    S1 => S3,
    /// ] { ... }
    ///
    /// EVENT2 [
    ///    S2 => S4,
    /// ] { ... }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut transitions: Vec<Transition> = Vec::new();

        while !input.is_empty() {
            let transition = Transition::parse(input)?;
            transitions.push(transition)
        }

        Ok(Transitions(transitions))
    }
}

impl ToTokens for Transitions {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for transition in &self.0 {
            transition.to_tokens(tokens);
        }

        let event_cases: Vec<_> = self
            .0
            .iter()
            .map(|v| EventCase {
                event_name: v.event_name.clone(),
                pairs: v.pairs.clone(),
                block: v.block.clone(),
            })
            .collect();

        tokens.extend(quote! {
            impl Machine {
                fn event(&mut self, event: Event) -> Result<bool, &'static str> {
                    match event {
                        #( #event_cases )*
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_transition_parse_and_to_tokens() {
        let transition: Transition = syn::parse2(quote! {
            EVENT1 [
               S1 => S2,
               S1 => S3,
            ] { }
        })
        .unwrap();

        let left = quote! {
            mod event1 {
                pub enum AfterExitS1 {
                    S2,
                    S3,
                }
                pub trait Callback {
                    fn on_event1(&self, data: (&str)) -> Result<(), &'static str>;
                    fn exit_s1(&self, data: (&str)) -> Result<AfterExitS1, &'static str>;
                    fn entry_s2_from_s1(&self, data: (&str));
                    fn entry_s3_from_s1(&self, data: (&str));
                }
            }
        };

        let mut right = TokenStream::new();
        transition.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_transitions_parse_and_to_tokens() {
        let transitions: Transitions = syn::parse2(quote! {
            EVENT1 [
               S1 => S2,
               S1 => S3,
            ] { }

            EVENT2 [
               S2 => S4,
            ] { }
        })
        .unwrap();

        let left = quote! {
            mod event1 {
                pub enum AfterExitS1 {
                    S2,
                    S3,
                }
                pub trait Callback {
                    fn on_event1(&self, data: (&str)) -> Result<(), &'static str>;
                    fn exit_s1(&self, data: (&str)) -> Result<AfterExitS1, &'static str>;
                    fn entry_s2_from_s1(&self, data: (&str));
                    fn entry_s3_from_s1(&self, data: (&str));
                }
            }
            mod event2 {
                pub enum AfterExitS2 {
                    S4,
                }
                pub trait Callback {
                    fn on_event2(&self, data: (&str)) -> Result<(), &'static str>;
                    fn exit_s2(&self, data: (&str)) -> Result<AfterExitS2, &'static str>;
                    fn entry_s4_from_s2(&self, data: (&str));
                }
            }
            impl Machine {
                fn event(&mut self, event: Event) -> Result<bool, &'static str> {
                    match event {
                        Event::EVENT1 => {
                            if let Err(err) = self.on_event1() {
                                return Err(err);
                            }
                            match self.current_state {
                                State::S1 => {
                                    match self.exit_s1() {
                                        Ok(r) => {
                                            match r {
                                                AfterExitS1::S2 => {
                                                    self.current_state = State::S2;
                                                    self.entry_s2_from_s1();
                                                    Ok(true)
                                                }
                                                AfterExitS1::S3 => {
                                                    self.current_state = State::S3;
                                                    self.entry_s3_from_s1();
                                                    Ok(true)
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            Err(err)
                                        }
                                    }
                                }
                            }
                        }
                        Event::EVENT2 => {
                            if let Err(err) = self.on_event2() {
                                return Err(err);
                            }
                            match self.current_state {
                                State::S2 => {
                                    match self.exit_s2() {
                                        Ok(r) => {
                                            match r {
                                                AfterExitS2::S4 => {
                                                    self.current_state = State::S4;
                                                    self.entry_s4_from_s2();
                                                    Ok(true)
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            Err(err)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        let mut right = TokenStream::new();
        transitions.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
