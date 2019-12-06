use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_quote, Ident,
};

use crate::fsm::{
    event::Event, initial_state::InitialState, state::State, transitions::Transitions,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Machine {
    pub state: State,
    pub event: Event,
    pub initial_state: InitialState,
    pub transitions: Transitions,
}

impl Parse for Machine {
    /// example machine tokens:
    ///
    /// ```text
    /// States { ... }
    ///
    /// InitialState( ... )
    ///
    /// Events { ... }
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
        // `State { ... }`
        let state = State::parse(input)?;

        // `InitialState ( ... )`
        let initial_state = InitialState::parse(input)?;

        // `Events { ... }`
        let event = Event::parse(input)?;

        // `EVENT1 [
        //    S1 => S2,
        //    S1 => S3
        // ] { ... }`
        let transitions = Transitions::parse(input)?;

        Ok(Machine {
            state,
            event,
            initial_state,
            transitions,
        })
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let initial_state = &self.initial_state;
        let state = &self.state;
        let event = &self.event;
        let transitions = &self.transitions;

        tokens.extend(quote! {
            #[allow(non_snake_case)]

            #state
            #initial_state
            #event

            pub struct Machine {
                current_state: State,
            }

            impl Machine {
                pub fn state(&self) -> State {
                    self.current_state
                }
            }

            #transitions

            pub fn new() -> Machine {
                Machine {
                    current_state: INIT_STATE,
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::state::State;
    use crate::fsm::transitions::TransitionPair;
    use crate::fsm::{initial_state::InitialState, transitions::Transition};
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote, ItemEnum, Visibility};

    #[test]
    fn test_machine_parse_and_to_tokens() {
        let machine: Machine = syn::parse2(quote! {
           #[derive(Clone, Copy, Debug)]
           pub enum State {
               Open,
               Close,
           }

           InitialState (Open)

           #[derive(Clone, Copy, Debug)]
           pub enum Event {
               Turn,
           }

           Turn [
               Open => Close,
               Close => Open,
               Close => Close,
           ] { }
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
