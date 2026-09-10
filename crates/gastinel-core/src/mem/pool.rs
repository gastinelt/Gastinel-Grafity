//! Generational Object Pool for stable, zero-allocation resource handles.
//!
//! Windows, surfaces, layers, and timers require stable IDs. A simple index is dangerous
//! because when an entity is destroyed and its index reused, stale events could affect
//! a completely different entity (the ABA problem).
//!
//! `Pool<T>` pairs each slot with a 32-bit generation counter. Lookups verify that the
//! handle generation matches the slot's current generation in O(1) time.

use std::fmt;

/// Strongly-typed 64-bit handle combining a 32-bit index and a 32-bit generation counter.
pub struct Handle<T> {
    pub index: u32,
    pub generation: u32,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> Handle<T> {
    pub const NULL: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: std::marker::PhantomData,
    };

    pub fn is_null(&self) -> bool {
        self.index == u32::MAX
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle(idx: {}, gen: {})", self.index, self.generation)
    }
}

enum Slot<T> {
    Occupied { value: T, generation: u32 },
    Free { next_free: Option<u32>, generation: u32 },
}

/// Generational object pool with O(1) allocation, O(1) lookup, and O(1) deallocation.
pub struct Pool<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
    active_count: usize,
}

impl<T> Pool<T> {
    /// Creates a pool preallocated to the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        for i in 0..capacity {
            slots.push(Slot::Free {
                next_free: if i + 1 < capacity { Some((i + 1) as u32) } else { None },
                generation: 1,
            });
        }

        Self {
            slots,
            free_head: if capacity > 0 { Some(0) } else { None },
            active_count: 0,
        }
    }

    /// Inserts an item into the pool and returns a generational handle.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        self.active_count += 1;

        if let Some(index) = self.free_head {
            let (next_free, generation) = match &self.slots[index as usize] {
                Slot::Free { next_free, generation } => (*next_free, *generation),
                Slot::Occupied { .. } => unreachable!("free_head pointed to occupied slot"),
            };

            self.free_head = next_free;
            self.slots[index as usize] = Slot::Occupied { value, generation };

            Handle {
                index,
                generation,
                _marker: std::marker::PhantomData,
            }
        } else {
            // Grow pool
            let index = self.slots.len() as u32;
            let generation = 1;
            self.slots.push(Slot::Occupied { value, generation });

            Handle {
                index,
                generation,
                _marker: std::marker::PhantomData,
            }
        }
    }

    /// Retrieves an immutable reference to an item if the handle is valid.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index as usize)?;
        match slot {
            Slot::Occupied { value, generation } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    /// Retrieves a mutable reference to an item if the handle is valid.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        match slot {
            Slot::Occupied { value, generation } if *generation == handle.generation => Some(value),
            _ => None,
        }
    }

    /// Removes an item from the pool. Returns the item if the handle was valid.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        match slot {
            Slot::Occupied { generation, .. } if *generation == handle.generation => {
                let old_gen = *generation;
                let new_gen = old_gen.wrapping_add(1).max(1); // avoid 0
                let next_free = self.free_head;

                let old_slot = std::mem::replace(
                    slot,
                    Slot::Free {
                        next_free,
                        generation: new_gen,
                    },
                );

                self.free_head = Some(handle.index);
                self.active_count -= 1;

                match old_slot {
                    Slot::Occupied { value, .. } => Some(value),
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }

    /// Returns the number of currently active items.
    pub fn len(&self) -> usize {
        self.active_count
    }

    pub fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    /// Iterate over all active items.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| match slot {
            Slot::Occupied { value, generation } => Some((
                Handle {
                    index: i as u32,
                    generation: *generation,
                    _marker: std::marker::PhantomData,
                },
                value,
            )),
            Slot::Free { .. } => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_insert_get_remove() {
        let mut pool = Pool::with_capacity(2);
        let h1 = pool.insert("Window 1");
        let h2 = pool.insert("Window 2");

        assert_eq!(pool.get(h1), Some(&"Window 1"));
        assert_eq!(pool.get(h2), Some(&"Window 2"));

        assert_eq!(pool.remove(h1), Some("Window 1"));
        assert_eq!(pool.get(h1), None); // stale handle

        // Next insert reuses h1 slot but with incremented generation
        let h3 = pool.insert("Window 3");
        assert_eq!(h3.index, h1.index);
        assert_ne!(h3.generation, h1.generation);
        assert_eq!(pool.get(h1), None);
        assert_eq!(pool.get(h3), Some(&"Window 3"));
    }
}
