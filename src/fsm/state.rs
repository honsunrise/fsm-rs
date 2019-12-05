use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, ItemEnum, Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct State(pub ItemEnum);

impl Parse for State {
    /// example states:
    ///
    /// ```text
    /// #[derive(Debug, PartialEq)]
    /// pub enum State<A, B> { A(A), B(B) }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let states = ItemEnum::parse(input)?;

        if states.ident != "State" {
            return Err(input.error("expected state enum define"));
        }

        Ok(State(states))
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let state = &self.0;
        tokens.extend(quote!(
            #state
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_state_parse() {
        let left: State = syn::parse2(quote! {
            #[derive(Debug, PartialEq)]
            pub enum State {
                Turn(&str),
            }
        })
        .unwrap();
        let right: State = State(parse_quote! {
            #[derive(Debug, PartialEq)]
            pub enum State {
                Turn(&str),
            }
        });

        assert_eq!(left, right);
    }

    #[test]
    fn test_state_to_tokens() {
        let state = State(parse_quote! {
            #[derive(Debug, PartialEq)]
            pub enum State {
                Turn(&str),
            }
        });

        let left = quote! {
            #[derive(Debug, PartialEq)]
            pub enum State {
                Turn(&str),
            }
        };

        let mut right = TokenStream::new();
        state.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
