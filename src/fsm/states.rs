use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, ItemEnum, Token, Type,
};

#[derive(Debug, PartialEq)]
pub(crate) struct State {
    pub state_name: Ident,
    pub state_type: Type,
}

impl Parse for State {
    /// example state:
    ///
    /// ```text
    /// S1 = S1
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// S1 = S1
        /// __
        let state_name: Ident = Ident::parse(input)?;

        /// S1 = S1
        ///    _
        let _: Token![=] = input.parse()?;

        /// S1 = S1
        ///      __
        let state_type: Type = Type::parse(input)?;

        Ok(State {
            state_name,
            state_type,
        })
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let state_name = &self.state_name;
        let state_type = &self.state_type;
        tokens.extend(quote!(
            #state_name(#state_type)
        ));
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct States(Vec<State>);

impl Parse for States {
    /// example states:
    ///
    /// ```text
    /// States {
    ///     S1 = S1,
    ///     S2 = S2,
    ///     S3 = S3,
    ///     S4 = S4,
    ///     S5 = S5
    /// }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// States { ... }
        /// -----------
        let states_magic = Ident::parse(input)?;

        if states_magic != "States" {
            return Err(input.error("expected States { ... }"));
        }

        let content;
        braced!(content in input);

        let mut transitions: Vec<State> = Vec::new();

        let states: Punctuated<State, Token![,]> = content.parse_terminated(State::parse)?;
        Ok(States(states.into_iter().collect()))
    }
}

impl ToTokens for States {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let states = &self.0;
        tokens.extend(quote!(
            #[derive(Clone, Debug, PartialEq)]
            pub enum State {
                #(#states),*
            }
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_states_parse_and_to_tokens() {
        let states: States = syn::parse2(quote! {
            States {
                S1 = S1,
                S2 = S2
            }
        })
        .unwrap();

        let left = quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum State {
                S1(S1),
                S2(S2)
            }
        };

        let mut right = TokenStream::new();
        states.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
