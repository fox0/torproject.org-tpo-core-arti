//! Utility module to safely refer to a mutable Arc.

use std::sync::{Arc, RwLock};

use crate::{Error, Result};

/// A shareable mutable-ish optional reference to a an [`Arc`].
///
/// Because you can't actually change a shared [`Arc`], this type implements
/// mutability by replacing the Arc itself with a new value.  It tries
/// to avoid needless clones by taking advantage of [`Arc::new_mut`].
///
// We give this construction its own type to simplify its users, and make
// sure we don't hold the lock against any async suspend points.
#[derive(Debug)]
pub(crate) struct SharedMutArc<T> {
    /// Locked reference to the current value.
    ///
    /// (It's okay to use RwLock here, because we never suspend
    /// while holding the lock.)
    dir: RwLock<Option<Arc<T>>>,
}

impl<T> Default for SharedMutArc<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SharedMutArc<T> {
    /// Construct a new empty SharedMutArc.
    pub fn new() -> Self {
        SharedMutArc {
            dir: RwLock::new(None),
        }
    }

    /// Replace the current value with `new_val`.
    pub fn replace(&self, new_val: T) {
        let mut w = self.dir.write().unwrap();
        *w = Some(Arc::new(new_val));
    }

    /// Remove the current value of this SharedMutArc.
    #[allow(unused)]
    pub fn clear(&self) {
        let mut w = self.dir.write().unwrap();
        *w = None;
    }

    /// Return a new reference to the current value, if there is one.
    pub fn get(&self) -> Option<Arc<T>> {
        let r = self.dir.read().unwrap();
        r.as_ref().map(Arc::clone)
    }

    /// Replace the contents of this SharedMutArc with the results of applying
    /// `func` to the inner value.
    ///
    /// Gives an error if there is no inner value.
    ///
    /// Other threads will not abe able to access the inner value
    /// while the function is running.
    ///
    /// # Limitation: No panic-safety
    ///
    /// If `func` panics while it's running, this object will become invalid
    /// and future attempts to use it will panic. (TODO: Fix this.)
    // Note: If we decide to make this type public, we'll probably
    // want to fiddle with how we handle the return type.
    pub fn mutate<F, U>(&self, func: F) -> Result<U>
    where
        F: FnOnce(&mut T) -> Result<U>,
        T: Clone,
    {
        match self.dir.write().unwrap().as_mut() {
            None => Err(Error::DirectoryNotPresent.into()), // Kinda bogus.
            Some(arc) => func(Arc::make_mut(arc)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn shared_mut_arc() {
        let val: SharedMutArc<Vec<u32>> = SharedMutArc::new();
        assert_eq!(val.get(), None);

        val.replace(Vec::new());
        assert_eq!(val.get().unwrap().as_ref()[..], []);

        val.mutate(|v| {
            v.push(99);
            Ok(())
        })
        .unwrap();
        assert_eq!(val.get().unwrap().as_ref()[..], [99]);

        val.clear();
        assert_eq!(val.get(), None);

        assert!(val
            .mutate(|v| {
                v.push(99);
                Ok(())
            })
            .is_err());
    }
}
