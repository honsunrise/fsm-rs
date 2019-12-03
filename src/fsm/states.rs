use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct States(pub Vec<State>);

impl Parse for States {
    /// example initial states tokens:
    ///
    /// ```text
    /// States { Open, Close }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut states: Vec<State> = Vec::new();

        // `States { ... }`
        //  ^^^^^^
        let block_name: Ident = input.parse()?;

        if block_name != "States" {
            return Err(input.error("expected `States { ... }`"));
        }

        // `States { ... }`
        //           ^^^
        let block_states;
        braced!(block_states in input);

        // `States { Open, Close }`
        //           ^^^^  ^^^^^
        let punctuated_states: Punctuated<Ident, Token![,]> =
            block_states.parse_terminated(Ident::parse)?;

        for name in punctuated_states {
            states.push(State { name });
        }

        Ok(States(states))
    }
}

impl ToTokens for States {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let states = &self.0;

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug)]
            pub enum State {
                #(#states,)*
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct State {
    pub name: Ident,
}

impl Parse for State {
    /// example state tokens:
    ///
    /// ```text
    /// Open
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;

        Ok(State { name })
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;

        tokens.extend(quote! {
            #name
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_state_parse() {
        let left: State = syn::parse2(quote! { Open }).unwrap();
        let right = State {
            name: parse_quote! { Open },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_state_to_tokens() {
        let state = State {
            name: parse_quote! { Open },
        };

        let left = quote! {
            Open
        };

        let mut right = TokenStream::new();
        state.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_states_to_tokens() {
        let states = States(vec![
            State {
                name: parse_quote! { Open },
            },
            State {
                name: parse_quote! { Close },
            },
        ]);

        let left = quote! {
            #[derive(Clone, Copy, Debug)]
            pub enum State {
                Open, Close,
            };
        };

        let mut right = TokenStream::new();
        states.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
