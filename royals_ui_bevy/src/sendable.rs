use std::sync::mpsc::{Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Sendable<T> {
    object: Arc<Mutex<T>>,
}

impl<T> Sendable<T> {
    pub fn new(object: T) -> Self {
        Sendable {
            object: Arc::new(Mutex::new(object)),
        }
    }

    pub fn get(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, T>, PoisonError<std::sync::MutexGuard<'_, T>>> {
        self.object.lock()
    }
}

impl<T> Sendable<Receiver<T>> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.get().unwrap().try_recv()
    }
}

impl<T> Sendable<Sender<T>> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        self.get().unwrap().send(t)
    }
}
