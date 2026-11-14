//! Stop-before-free ownership, usable with a fake owner to test ordering without a native crash.
use std::sync::{
    Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
#[derive(Debug)]
struct State<T> {
    owner: Option<T>,
    active: bool,
}
#[derive(Debug)]
pub struct Generator<T> {
    state: Mutex<State<T>>,
    idle: Condvar,
    stopped: AtomicBool,
    generation: AtomicU64,
}
#[derive(Debug)]
pub struct Lease<'a, T> {
    generator: &'a Generator<T>,
    owner: Option<T>,
    generation: u64,
}
impl<T> Generator<T> {
    pub fn new(owner: T) -> Self {
        Self {
            state: Mutex::new(State {
                owner: Some(owner),
                active: false,
            }),
            idle: Condvar::new(),
            stopped: AtomicBool::new(false),
            generation: AtomicU64::new(0),
        }
    }
    pub fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
    pub fn close(&self) {
        self.stopped.store(true, Ordering::Release);
        self.cancel();
        let owner = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .owner
            .take();
        drop(owner);
        self.idle.notify_all();
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
    pub fn lease(&self) -> Result<Lease<'_, T>, &'static str> {
        let generation = self.generation.load(Ordering::Acquire);
        let mut state = self.state.lock().map_err(|_| "Generator poisoned")?;
        while state.active && !self.stopped() {
            state = self
                .idle
                .wait(state)
                .map_err(|_| "Generator wait poisoned")?;
        }
        if self.stopped() {
            return Err("Generator closed");
        }
        let owner = state.owner.take().ok_or("Generator missing owner")?;
        state.active = true;
        Ok(Lease {
            generator: self,
            owner: Some(owner),
            generation,
        })
    }
}
impl<T> Lease<'_, T> {
    pub fn owner(&self) -> &T {
        self.owner.as_ref().expect("Live generator lease")
    }
    pub fn current(&self) -> bool {
        !self.generator.stopped()
            && self.generator.generation.load(Ordering::Acquire) == self.generation
    }
}
impl<T> Drop for Lease<'_, T> {
    fn drop(&mut self) {
        let mut state = self
            .generator
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !self.generator.stopped() {
            state.owner = self.owner.take();
        }
        state.active = false;
        self.generator.idle.notify_all();
    }
}
impl<T> Drop for Generator<T> {
    fn drop(&mut self) {
        self.close();
    }
}
#[cfg(test)]
mod tests;
