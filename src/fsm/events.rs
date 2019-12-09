use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, ItemEnum, Token, Type,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Event {
    pub event_name: Ident,
    pub event_type: Type,
}

impl Parse for Event {
    /// example event:
    ///
    /// ```text
    /// S1 = S1
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// S1 = S1
        /// __
        let event_name: Ident = Ident::parse(input)?;

        /// S1 = S1
        ///    _
        let _: Token![=] = input.parse()?;

        /// S1 = S1
        ///      __
        let event_type: Type = Type::parse(input)?;

        Ok(Event {
            event_name,
            event_type,
        })
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event_name = &self.event_name;
        let event_type = &self.event_type;
        tokens.extend(quote!(
            #event_name(#event_type)
        ));
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Events(pub Vec<Event>);

impl Parse for Events {
    /// example events:
    ///
    /// ```text
    /// Events {
    ///     S1 = S1,
    ///     S2 = S2,
    ///     S3 = S3,
    ///     S4 = S4,
    ///     S5 = S5
    /// }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// Events { ... }
        /// --------------
        let events_magic = Ident::parse(input)?;

        if events_magic != "Events" {
            return Err(input.error("expected Events { ... }"));
        }

        let content;
        braced!(content in input);

        let mut transitions: Vec<Event> = Vec::new();

        let events: Punctuated<Event, Token![,]> = content.parse_terminated(Event::parse)?;
        Ok(Events(events.into_iter().collect()))
    }
}

impl ToTokens for Events {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let events = &self.0;
        tokens.extend(quote!(
            #[derive(Clone, Debug, PartialEq)]
            pub enum Event {
                #(#events),*
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
    fn test_events_parse_and_to_tokens() {
        let events: Events = syn::parse2(quote! {
            Events {
                E1 = E1
            }
        })
        .unwrap();

        let left = quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum Event {
                E1(E1)
            }
        };

        let mut right = TokenStream::new();
        events.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
