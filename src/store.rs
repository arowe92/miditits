use std::sync::{Arc, Mutex, MutexGuard};

use futures::future::BoxFuture;

///
/// ### Base Store class
///
#[derive(Clone)]
pub struct AsyncStore<State, Event, AsyncEvent>
where
    State: Clone + Send + Sync,
    Event: Clone,
    AsyncEvent: Clone,
{
    state: Arc<Mutex<State>>,
    events: Arc<Mutex<Vec<Event>>>,
    is_working: Arc<Mutex<i32>>,
    reducer: fn(&mut State, Event),
    async_reducer: AsyncReducer<AsyncEvent, Self>
}

type AsyncReducer<AsyncEvent, Store> = fn(
        event: AsyncEvent,
        store: Store,
    ) -> BoxFuture<'static, ()>;

impl<State, Event, AsyncEvent> AsyncStore<State, Event, AsyncEvent>
where
    State: Clone + Send + Sync,
    Event: Clone,
    AsyncEvent: Clone
{
    pub fn new(s: State, reducer: fn(&mut State, Event), async_reducer: AsyncReducer<AsyncEvent, Self>) -> Self {
        Self {
            state: Arc::new(Mutex::new(s)),
            events: Arc::new(Mutex::new(vec![])),
            is_working: Arc::new(Mutex::new(0)),
            reducer,
            async_reducer,
        }
    }

    fn events(&self) -> MutexGuard<Vec<Event>> {
        self.events.lock().unwrap()
    }

    pub fn state(&self) -> MutexGuard<State> {
        self.state.lock().unwrap()
    }

    pub fn mutate<F>(&self, mut f: F) where F: FnMut(&mut State) {
        f(&mut *self.state());
    }

    pub fn dispatch(&self, e: Event) {
        self.events().push(e);
        self.update();
    }

    pub async fn dispatch_async(&self, event: AsyncEvent) {
        *self.is_working.lock().unwrap() += 1;
        (self.async_reducer)(event, self.clone()).await;
        *self.is_working.lock().unwrap() -= 1;
    }

    pub fn is_working(&self) -> i32 {
        *self.is_working.lock().unwrap()
    }

    fn update(&self) {
        // Clone State
        let mut lock = self.state();

        let state = &mut *lock;

        // Drain Events and update state
        for event in self.events().drain(..) {
            (self.reducer)(state, event);
        }

        drop(lock);
    }

    pub fn get_state(&self) -> State {
        self.state().clone()
    }
}

struct HasChanges(bool);

pub trait Slice: Sync + Send + Clone {
    type Event: Sync + Send + Clone + 'static;
    type AsyncEvent: Sync + Send + Clone;

    fn reducer(state: &mut Self, event: Self::Event);
    fn async_reducer(
        event: Self::AsyncEvent,
        store: AsyncStore<Self, Self::Event, Self::AsyncEvent>,
    ) -> BoxFuture<'static, ()>;
}

pub struct Store<T>
where
    T: Slice + 'static,
{
    pub rt: tokio::runtime::Runtime,
    store: AsyncStore<T, T::Event, T::AsyncEvent>,
    has_changes: Arc<Mutex<HasChanges>>,
}

impl<T> Store<T>
where
    T: Slice + 'static,
{
    pub fn new(s: T) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Self {
            rt,
            has_changes: Arc::new(Mutex::new(HasChanges(false))),
            store: AsyncStore::new(s, T::reducer, T::async_reducer),
        }
    }

    #[allow(dead_code)]
    fn update(&mut self) {
        self.store.update();
        self.has_changes.lock().unwrap().0 = true;
    }

    pub fn dispatch(&self, e: T::Event) {
        self.store.dispatch(e);
    }

    pub fn get_state(&self) -> T {
        self.store.get_state()
    }

    pub fn drain_changes(&mut self) -> bool {
        let mut lock = self.has_changes.lock().unwrap();
        let ret = lock.0;
        lock.0 = false;
        ret
    }

    pub fn dispatch_async(&self, event: T::AsyncEvent) {
        let store = self.store.clone();
        self.rt.spawn(async move {
            store.dispatch_async(event).await
        });
    }

    pub fn is_working(&self) -> bool {
        self.store.is_working() > 0
    }
}

trait Action: Clone + 'static {
    type ActionEnum: Clone;
    type ActionMeta: Clone;

    fn action() -> Self::ActionEnum;
    fn meta() -> Self::ActionMeta;
}


// ===============================
//     TESTS
// ===============================
#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[derive(Clone)]
    struct TestState {
        a: i32,
    }

    impl Slice for TestState {
        type Event = TestEvent;
        type AsyncEvent = TestAsyncEvent;

        fn reducer(s: &mut TestState, e: TestEvent) {
            match e {
                TestEvent::Add(x) => s.a += x,
                TestEvent::Set(x) => s.a = x,
            };
        }

        fn async_reducer(
            event: Self::AsyncEvent,
            store: AsyncStore<Self, Self::Event, Self::AsyncEvent>,
        ) -> BoxFuture<'static, ()> {
            match event {
                TestAsyncEvent::Add(x) => Box::pin(async move { store.dispatch(TestEvent::Add(x)) }),
                TestAsyncEvent::Set(x) => Box::pin(async move { store.dispatch(TestEvent::Set(x)) }),
            }
        }
    }

    #[derive(Clone)]
    enum TestEvent {
        Add(i32),
        Set(i32),
    }

    #[derive(Clone)]
    enum TestAsyncEvent {
        Add(i32),
        Set(i32),
    }

    impl Default for TestState {
        fn default() -> TestState {
            TestState { a: 0 }
        }
    }
}
