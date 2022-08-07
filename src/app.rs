use std::{
    cmp::{max, min},
    path::Path,
    sync::Arc,
};

use egui_glium::EguiGlium;
use futures::future::BoxFuture;

use crate::{app_state::*, store::Slice, store::Store as StoreBase};

// Import App events
pub use crate::app_state::AppEvent;
use AppEvent::*;

// Declare Specfic store type
type Store = StoreBase<AppState>;

/// App Structure
/// Can hold Stateful Managers / Contexts
/// Midi manager probably goes here
pub struct App {
    last_state: AppState,
}

/// Responses this widget can have
pub enum AppResponse {
    Redraw,
    None,
}

impl App {
    pub fn new() -> Self {
        Self {
            last_state: AppState::default(),
        }
    }

    pub fn handle_key_event(&mut self, store: &Store, event: glium::glutin::event::KeyboardInput) {
        use glium::glutin::event::VirtualKeyCode::*;

        if event.virtual_keycode == None {
            return;
        }

        // Handle Jet events and dispatch to store
        #[allow(deprecated)]
        match (event.virtual_keycode.unwrap(), event.modifiers) {
            (Up, _) => store.dispatch(AppEvent::AddNumber(1)),
            (Down, _) => store.dispatch(AppEvent::AddNumber(-1)),
            _ => (),
        };
    }

    ///
    /// Main Render Function
    ///
    pub fn render(
        &mut self,
        store: &Store,
        egui: &mut EguiGlium,
        display: &glium::Display,
    ) -> AppResponse {
        // Get Current State
        let state = store.get_state();

        egui.run(display, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if ui.button("add 1").clicked() {
                    store.dispatch(AppEvent::AddNumber(1));
                }

                if ui.button("sub 1 delayed").clicked() {
                    store.dispatch_async(AsyncAppEvent::DelayAddNumber(-1));
                }

                ui.label(format!("{}", state.midi));
            });
        });

        // Cache Last State
        self.last_state = state;

        AppResponse::None
    }
}
