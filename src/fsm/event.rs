use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, ItemEnum, Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Event(pub ItemEnum);

impl Parse for Event {
    /// example events:
    ///
    /// ```text
    /// #[derive(Debug, PartialEq)]
    /// pub enum Event<A, B> { A(A), B(B) }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let events = ItemEnum::parse(input)?;

        if events.ident != "Event" {
            return Err(input.error("expected event enum define"));
        }

        Ok(Event(events))
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event = &self.0;
        tokens.extend(quote!(
            #event
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_event_parse() {
        let left: Event = syn::parse2(quote! {
            #[derive(Debug, PartialEq)]
            pub enum Event {
                Turn(&str),
            }
        })
        .unwrap();
        let right: Event = Event(parse_quote! {
            #[derive(Debug, PartialEq)]
            pub enum Event {
                Turn(&str),
            }
        });

        assert_eq!(left, right);
    }

    #[test]
    fn test_event_to_tokens() {
        let event = Event(parse_quote! {
            #[derive(Debug, PartialEq)]
            pub enum Event {
                Turn(&str),
            }
        });

        let left = quote! {
            #[derive(Debug, PartialEq)]
            pub enum Event {
                Turn(&str),
            }
        };

        let mut right = TokenStream::new();
        event.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
