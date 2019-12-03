use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    Ident,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InitialState {
    pub name: Ident,
}

impl Parse for InitialState {
    /// example initial state tokens:
    ///
    /// ```text
    /// InitialState(Open)
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // `InitialState ( ... )`
        //  ^^^^^^^^^^^^^
        let magic_name: Ident = input.parse()?;

        if magic_name != "InitialState" {
            return Err(input.error("expected `InitialState ( ... )`"));
        }

        // `InitialStates ( ... )`
        //                  ^^^
        let initial_state;
        parenthesized!(initial_state in input);

        // `InitialStates ( Locked )`
        //                  ^^^^^^
        let name: Ident = initial_state.parse()?;

        Ok(InitialState { name })
    }
}

impl ToTokens for InitialState {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;

        tokens.extend(quote! {
            const INIT_STATE: State = State::#name;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse2, parse_quote};

    #[test]
    fn test_initial_state_parse() {
        let left: InitialState = parse2(quote! {
            InitialState { Open, Close }
        })
        .unwrap();

        let right = InitialState {
            name: parse_quote! { Open },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_initial_state_to_tokens() {
        let initial_state = InitialState {
            name: parse_quote! { Open },
        };

        let left = quote! {
            const INIT_STATE: State = State::Open;
        };

        let mut right = TokenStream::new();
        initial_state.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
