//! Unique atomic identifiers.

use std::{
    marker::PhantomData,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};

/// An identifier allocated from an [`IdPool`].
pub struct Id<T> {
    inner: Arc<usize>,
    recycled_ids: Arc<Mutex<Vec<Id<T>>>>,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            recycled_ids: self.recycled_ids.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T> Drop for Id<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) <= 1 {
            let mut guard = self
                .recycled_ids
                .lock()
                .expect("Cannot lock recycled_ids for recycling");
            guard.push(self.clone());
        }
    }
}

/// A pool from which to allocate identifiers.
///
/// [`Id`]s allocated this way are unique within the pool.
pub struct IdPool<T> {
    next_k: AtomicUsize,
    recycled_ids: Arc<Mutex<Vec<Id<T>>>>,
    _phantom: PhantomData<T>,
}

impl<T> Default for IdPool<T> {
    fn default() -> Self {
        Self {
            next_k: Default::default(),
            recycled_ids: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T> IdPool<T> {
    /// Dequeue an [`Id`] from the pool.
    ///
    /// [`Id`]s acquired this way can simply be dropped to return them to the pool.
    pub fn get_id(&self) -> Id<T> {
        let mut guard = self.recycled_ids.lock().expect("Cannot lock recycled_ids");
        if let Some(id) = guard.pop() {
            return id;
        }

        let next_k = self
            .next_k
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Id {
            inner: Arc::new(next_k),
            recycled_ids: self.recycled_ids.clone(),
            _phantom: PhantomData,
        }
    }
}
