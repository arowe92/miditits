use std::{
    cmp::{max, min},
    path::Path,
    sync::Arc,
};

use egui_glium::EguiGlium;
use futures::future::BoxFuture;

// Managers
use crate::{store::Slice, store::Store as StoreBase};

type Store = StoreBase<AppState>;

/// App GUI State
#[derive(Debug, Clone)]
pub struct AppState {
    pub midi: i32,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            midi: 0,
        }
    }
}

/// Events to the app state
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum AppEvent {
    Initialize,
    AddNumber(i32),
}

#[derive(Clone, Debug)]
pub enum AsyncAppEvent {
    DelayAddNumber(i32),
}

impl Slice for AppState {
    type Event = AppEvent;
    type AsyncEvent = AsyncAppEvent;

    /// App State Reducer
    fn reducer(state: &mut Self, event: Self::Event) {
        use AppEvent::*;
        let mut dispatch = Vec::<Self::Event>::new();

        match event {
            Initialize => {
                state.midi = 69;
            }
            AddNumber(num) => {
                state.midi += num;
            }
        }

        println!("Event: {:?}", &event);
        println!("State: {:?}", &state);
    }

    fn async_reducer(
        event: Self::AsyncEvent,
        store: crate::store::AsyncStore<Self, Self::Event, Self::AsyncEvent>,
    ) -> BoxFuture<'static, ()> {
        use AppEvent::*;

        Box::pin(async move {
            match event {
                AsyncAppEvent::DelayAddNumber(num) => {
                    tokio::time::sleep(std::time::Duration::from_secs_f32(1.0)).await;

                    store.dispatch(AppEvent::AddNumber(num));
                }
            }
        })
    }
}
