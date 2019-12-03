use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Events(pub Vec<Event>);

impl Parse for Events {
    /// example initial events tokens:
    ///
    /// ```text
    /// Events { TurnOn, TurnOff }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut events: Vec<Event> = Vec::new();

        // `Events { ... }`
        //  ^^^^^^
        let block_name: Ident = input.parse()?;

        if block_name != "Events" {
            return Err(input.error("expected `Events { ... }`"));
        }

        // `Events { ... }`
        //           ^^^
        let block_events;
        braced!(block_events in input);

        // `Events { Open, Close }`
        //           ^^^^  ^^^^^
        let punctuated_block_events: Punctuated<Ident, Token![,]> =
            block_events.parse_terminated(Ident::parse)?;

        for name in punctuated_block_events {
            events.push(Event { name });
        }

        Ok(Events(events))
    }
}

impl ToTokens for Events {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let events = &self.0;

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug)]
            pub enum Event {
                #(#events,)*
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Event {
    pub name: Ident,
}

impl Parse for Event {
    /// example state tokens:
    ///
    /// ```text
    /// TurnOn
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;

        Ok(Event { name })
    }
}

impl ToTokens for Event {
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
        let left: Event = syn::parse2(quote! { TurnOn }).unwrap();
        let right = Event {
            name: parse_quote! { TurnOn },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_state_to_tokens() {
        let state = Event {
            name: parse_quote! { TurnOn },
        };

        let left = quote! {
            TurnOn
        };

        let mut right = TokenStream::new();
        state.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_events_to_tokens() {
        let events = Events(vec![
            Event {
                name: parse_quote! { TurnOn },
            },
            Event {
                name: parse_quote! { TurnOff },
            },
        ]);

        let left = quote! {
            #[derive(Clone, Copy, Debug)]
            pub enum Event {
                TurnOn, TurnOff,
            };
        };

        let mut right = TokenStream::new();
        events.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
