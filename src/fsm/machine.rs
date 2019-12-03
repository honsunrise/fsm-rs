use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote, Ident,
};

use crate::fsm::{
    events::Events, initial_state::InitialState, states::States, transitions::Transitions,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Machine {
    pub states: States,
    pub events: Events,
    pub initial_state: InitialState,
    pub transitions: Transitions,
}

impl Parse for Machine {
    /// example machine tokens:
    ///
    /// ```text
    /// States { ... }
    ///
    /// Events { ... }
    ///
    /// InitialState( ... )
    ///
    /// EVENT1 [
    ///    S1 => S2,
    ///    S1 => S3
    /// ] { ... }
    ///
    /// EVENT2 [
    ///    S2 => S4,
    /// ] { ... }
    ///
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // `States { ... }`
        let states = States::parse(input)?;

        // `Events { ... }`
        let events = Events::parse(input)?;

        // `InitialState ( ... )`
        let initial_state = InitialState::parse(input)?;

        // `EVENT1 [
        //    S1 => S2,
        //    S1 => S3
        // ] { ... }`
        let transitions = Transitions::parse(input)?;

        Ok(Machine {
            states,
            events,
            initial_state,
            transitions,
        })
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let initial_state = &self.initial_state;
        let states = &self.states;
        let events = &self.events;
        let transitions = &self.transitions;

        tokens.extend(quote! {
            #[allow(non_snake_case)]

            #states
            #initial_state
            #events
            #transitions
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::events::Event;
    use crate::fsm::states::State;
    use crate::fsm::transitions::TransitionPair;
    use crate::fsm::{initial_state::InitialState, transitions::Transition};
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_machine_parse() {
        let left: Machine = syn::parse2(quote! {
           States {
               Open, Close,
           }

           Events {
               Turn,
           }

           InitialState (Open)

           Turn [
               Open => Close,
               Close => Open,
           ] { }
        })
        .unwrap();

        let right = Machine {
            states: States(vec![
                State {
                    name: parse_quote! { Open },
                },
                State {
                    name: parse_quote! { Close },
                },
            ]),
            events: Events(vec![Event {
                name: parse_quote! { Turn },
            }]),
            initial_state: InitialState {
                name: parse_quote! { Open },
            },
            transitions: Transitions(vec![Transition {
                event_name: parse_quote! { Turn },
                pairs: vec![
                    TransitionPair {
                        from: parse_quote! { Open },
                        to: parse_quote! { Close },
                    },
                    TransitionPair {
                        from: parse_quote! { Close },
                        to: parse_quote! { Open },
                    },
                ],
                block: parse_quote! { {} },
            }]),
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_machine_to_tokens() {
        let machine = Machine {
            states: States(vec![
                State {
                    name: parse_quote! { Open },
                },
                State {
                    name: parse_quote! { Close },
                },
            ]),
            events: Events(vec![Event {
                name: parse_quote! { Turn },
            }]),
            initial_state: InitialState {
                name: parse_quote! { Open },
            },
            transitions: Transitions(vec![Transition {
                event_name: parse_quote! { Turn },
                pairs: vec![
                    TransitionPair {
                        from: parse_quote! { Open },
                        to: parse_quote! { Close },
                    },
                    TransitionPair {
                        from: parse_quote! { Close },
                        to: parse_quote! { To },
                    },
                ],
                block: parse_quote! { {} },
            }]),
        };

        let left = quote! {
            #[allow(non_snake_case)]
            #[derive(Clone, Copy, Debug)]
            pub enum State {
                Open,
                Close,
            }
            const INIT_STATE: State = State::Open;
            #[derive(Clone, Copy, Debug)]
            pub enum Event {
                Turn,
            }
        };

        let mut right = TokenStream::new();
        machine.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
