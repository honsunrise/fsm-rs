use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote,
    token::{Colon, Pub},
    Field, Fields, Ident, ItemEnum, ItemStruct, Token, Type, VisPublic, Visibility,
};

use crate::fsm::{
    events::Events, machine_context::MachineContext, states::States,
    transitions::Transitions,
};
use syn::spanned::Spanned;

#[derive(Debug, PartialEq)]
pub(crate) struct Machine {
    pub machine_context: MachineContext,
    pub events: Events,
    pub states: States,
    pub transitions: Transitions,
}

impl Parse for Machine {
    /// example machine:
    ///
    /// ```text
    ///
    /// Context = Machine;
    ///
    /// States {
    ///     S1 = S1,
    ///     S2 = S2,
    ///     S3 = S3,
    ///     S4 = S4,
    ///     S5 = S5
    /// }
    ///
    /// InitialState( ... );
    ///
    /// Events {
    ///     EVENT1 = Event1,
    ///     EVENT2 = Event2
    /// }

    /// Transitions {
    ///     EVENT1 [
    ///        S1 => S2,
    ///        S1 => S3,
    ///     ],
    ///     EVENT2 [
    ///         S4 => S5,
    ///     ]
    /// }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        /// Context = Machine;
        let machine_context = MachineContext::parse(input)?;

        /// States {
        ///     S1 = S1,
        ///     S2 = S2,
        ///     S3 = S3,
        ///     S4 = S4,
        ///     S5 = S5
        /// }
        let states = States::parse(input)?;

        /// Events {
        ///     EVENT1 = Event1,
        ///     EVENT2 = Event2
        /// }
        let events = Events::parse(input)?;

        /// Transitions {
        ///     EVENT1 [
        ///         S1 => S2,
        ///         S1 => S3,
        ///     ],
        ///     EVENT2 [
        ///         S4 => S5,
        ///     ],
        /// }
        let transitions = Transitions::parse(input)?;

        Ok(Machine {
            machine_context,
            events,
            states,
            transitions,
        })
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let states = &self.states;
        let events = &self.events;

        let machine_context_type = &self.machine_context.context_type();

        let event_fn_impl = self.transitions.to_event_fn_tokens();

        tokens.extend(quote! {
            #[allow(non_snake_case)]

            #states

            #events

            pub struct Machine {
                context: #machine_context_type,
                current_state: State,
            }

            impl Machine {
                #event_fn_impl

                pub fn new() -> Machine {
                    Machine {
                        context: #machine_context_type::default(),
                        current_state: State::default(),
                    }
                }

                pub fn state(&self) -> State {
                    self.current_state
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote, ItemEnum, Visibility};

    #[test]
    fn test_machine_parse_and_to_tokens() {
        let machine: Machine = syn::parse2(quote! {
            Context = FSM;

            States {
                S1 = S1,
                S2 = S2,
                S3 = S3,
                S4 = S4,
                S5 = S5
            }

            Events {
                EVENT1 = Event1,
                EVENT2 = Event2
            }

            Transitions {
                EVENT1 [
                   S1 => S2,
                   S1 => S3,
                ],
                EVENT2 [
                    S4 => S5,
                ]
            }
        })
        .unwrap();

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
            pub struct Machine {
                current_state: State,
            }
            impl Machine {
                pub fn state(&self) -> State {
                    self.current_state
                }
            }
            mod turn {
                pub enum AfterExitClose {
                    Close,
                    Open,
                }
                pub enum AfterExitOpen {
                    Close,
                }
                pub trait Callback {
                    fn on_turn(&self, data: (&str)) -> Result<(), &'static str>;
                    fn exit_close(&self, data: (&str)) -> Result<AfterExitClose, &'static str>;
                    fn entry_close_from_close(&self, data: (&str));
                    fn entry_open_from_close(&self, data: (&str));
                    fn exit_open(&self, data: (&str)) -> Result<AfterExitOpen, &'static str>;
                    fn entry_close_from_open(&self, data: (&str));
                }
            }
            impl Machine {
                fn event(&mut self, event: Event) -> Result<bool, &'static str> {
                    match event {
                        Event::Turn => {
                            if let Err(err) = self.on_turn() {
                                return Err(err);
                            }
                            match self.current_state {
                                State::Close => {
                                    match self.exit_close() {
                                        Ok(r) => {
                                            match r {
                                                AfterExitClose::Close => {
                                                    self.current_state = State::Close;
                                                    self.entry_close_from_close();
                                                    Ok(true)
                                                }
                                                AfterExitClose::Open => {
                                                    self.current_state = State::Open;
                                                    self.entry_open_from_close();
                                                    Ok(true)
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            Err(err)
                                        }
                                    }
                                }
                                State::Open => {
                                    match self.exit_open() {
                                        Ok(r) => {
                                            match r {
                                                AfterExitOpen::Close => {
                                                    self.current_state = State::Close;
                                                    self.entry_close_from_open();
                                                    Ok(true)
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            Err(err)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            pub fn new() -> Machine {
                Machine {
                    current_state: INIT_STATE,
                }
            }
        };

        let mut right = TokenStream::new();
        machine.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right));
    }
}
