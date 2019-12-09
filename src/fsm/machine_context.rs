use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::token::{Colon, Pub};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_quote, Expr, Field, Fields, Ident, ItemStruct, Token, Type, VisPublic, Visibility,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MachineContext {
    context_type: Type,
}

impl Parse for MachineContext {
    /// example machine context:
    ///
    /// ```text
    /// Context = Machine;
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// Context = Machine;
        /// _______
        let context_magic: Ident = Ident::parse(input)?;

        /// Context = Machine;
        ///         _
        let _: Token![=] = input.parse()?;

        /// Context = Machine;
        ///           _______
        let context_type: Type = Type::parse(input)?;

        /// Context = Machine;
        ///                  _
        let _: Token![;] = input.parse()?;

        Ok(MachineContext { context_type })
    }
}

impl MachineContext {
    pub fn context_type(&self) -> Type {
        self.context_type.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse2, parse_quote};

    #[test]
    fn test_initial_state_parse() {
        let _: MachineContext = parse2(quote! {
            pub struct FSM {
                str: String
            }
        })
        .unwrap();
    }
}
