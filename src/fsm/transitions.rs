use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, ExprBlock, Ident, Stmt, Token,
};

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
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Transition {
    pub event_name: Ident,
    pub pairs: Vec<TransitionPair>,
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

        let mut transition_pairs: Vec<TransitionPair> = Vec::new();

        // EVENT1 [ S1 => S2, S1 => S3, ] { ... }
        //          ^^^^^^^^^^^^^^^^^^^
        let punctuated_block_transition: Punctuated<TransitionPair, Token![,]> =
            block_transition.parse_terminated(TransitionPair::parse)?;

        for pair in punctuated_block_transition {
            transition_pairs.push(pair);
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

        tokens.extend(quote! {});
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_transition_parse() {
        let left: Transition = syn::parse2(quote! {
            EVENT1 [
               S1 => S2,
               S1 => S3,
            ] { }
        })
        .unwrap();

        let right = Transition {
            event_name: parse_quote! { EVENT1 },
            pairs: vec![
                TransitionPair {
                    from: parse_quote! { S1 },
                    to: parse_quote! { S2 },
                },
                TransitionPair {
                    from: parse_quote! { S1 },
                    to: parse_quote! { S3 },
                },
            ],
            block: parse_quote! { {} },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_transition_to_tokens() {
        let transition = Transition {
            event_name: parse_quote! { EVENT1 },
            pairs: vec![
                TransitionPair {
                    from: parse_quote! { S1 },
                    to: parse_quote! { S2 },
                },
                TransitionPair {
                    from: parse_quote! { S1 },
                    to: parse_quote! { S3 },
                },
            ],
            block: parse_quote! { {} },
        };

        let left = quote! {};

        let mut right = TokenStream::new();
        transition.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_transitions_parse() {
        let left: Transitions = syn::parse2(quote! {
            EVENT1 [
               S1 => S2,
               S1 => S3,
            ] { }

            EVENT2 [
               S2 => S4,
            ] { }
        })
        .unwrap();

        let right = Transitions(vec![
            Transition {
                event_name: parse_quote! { EVENT1 },
                pairs: vec![
                    TransitionPair {
                        from: parse_quote! { S1 },
                        to: parse_quote! { S2 },
                    },
                    TransitionPair {
                        from: parse_quote! { S1 },
                        to: parse_quote! { S3 },
                    },
                ],
                block: parse_quote! { {} },
            },
            Transition {
                event_name: parse_quote! { EVENT2 },
                pairs: vec![TransitionPair {
                    from: parse_quote! { S2 },
                    to: parse_quote! { S4 },
                }],
                block: parse_quote! { {} },
            },
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn test_transitions_to_tokens() {
        let transitions = Transitions(vec![
            Transition {
                event_name: parse_quote! { EVENT1 },
                pairs: vec![
                    TransitionPair {
                        from: parse_quote! { S1 },
                        to: parse_quote! { S2 },
                    },
                    TransitionPair {
                        from: parse_quote! { S1 },
                        to: parse_quote! { S3 },
                    },
                ],
                block: parse_quote! { {} },
            },
            Transition {
                event_name: parse_quote! { EVENT2 },
                pairs: vec![TransitionPair {
                    from: parse_quote! { S2 },
                    to: parse_quote! { S4 },
                }],
                block: parse_quote! { {} },
            },
        ]);

        let left = quote! {};

        let mut right = TokenStream::new();
        transitions.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
