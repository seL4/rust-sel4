//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

extern crate alloc;

use alloc::vec::Vec;
use core::iter;
use core::mem;

type Result<T> = core::result::Result<T, SlotTrackerError>;

pub trait SlotStateTypes {
    type Common = ();
    type Free = ();
    type Occupied = ();
}

pub struct SlotTracker<T: SlotStateTypes> {
    entries: Vec<Entry<T>>,
    free_list_head_index: Option<usize>,
    num_free: usize,
}

struct Entry<T: SlotStateTypes> {
    common: T::Common,
    state: StateInternal<T>,
}

enum StateInternal<T: SlotStateTypes> {
    Free {
        free_list_next_index: Option<usize>,
        value: T::Free,
    },
    Occupied {
        value: T::Occupied,
    },
}

impl<T: SlotStateTypes> StateInternal<T> {
    fn project(&self) -> SlotState {
        match self {
            Self::Free { .. } => SlotState::Free,
            Self::Occupied { .. } => SlotState::Occupied,
        }
    }
}

impl<T: SlotStateTypes> SlotTracker<T> {
    pub fn new(iter: impl Iterator<Item = (T::Common, T::Free)>) -> Self {
        let mut entries = Vec::new();
        let mut free_list_head_index = None;
        entries.extend(iter.enumerate().map(|(i, (v_common, v_free))| Entry {
            common: v_common,
            state: StateInternal::Free {
                free_list_next_index: free_list_head_index.replace(i),
                value: v_free,
            },
        }));
        let num_free = entries.len();
        Self {
            entries,
            free_list_head_index,
            num_free,
        }
    }

    pub fn new_with_capacity(common: T::Common, free: T::Free, capacity: usize) -> Self
    where
        T: SlotStateTypes<Common: Clone, Free: Clone>,
    {
        Self::new(iter::repeat((common, free)).take(capacity))
    }

    pub fn new_occupied(iter: impl Iterator<Item = (T::Common, T::Occupied)>) -> Self {
        let entries = iter
            .map(|(v_common, v_occupied)| Entry {
                common: v_common,
                state: StateInternal::Occupied { value: v_occupied },
            })
            .collect();
        Self {
            entries,
            free_list_head_index: None,
            num_free: 0,
        }
    }

    pub fn new_occupied_with_capacity(
        common: T::Common,
        occupied: T::Occupied,
        capacity: usize,
    ) -> Self
    where
        T: SlotStateTypes<Common: Clone, Occupied: Clone>,
    {
        Self::new_occupied(iter::repeat((common, occupied)).take(capacity))
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn num_free(&self) -> usize {
        self.num_free
    }

    pub fn num_occupied(&self) -> usize {
        self.capacity() - self.num_free()
    }

    pub fn peek_next_free_index(&self) -> Option<usize> {
        self.free_list_head_index
    }

    pub fn peek_next_free_value(&self) -> Option<&T::Free> {
        let index = self.peek_next_free_index()?;
        Some(self.get_state_value(index).unwrap().as_free().unwrap())
    }

    fn get_entry(&self, index: usize) -> Result<&Entry<T>> {
        self.entries.get(index).ok_or(SlotTrackerError::OutOfBounds)
    }

    fn get_entry_mut(&mut self, index: usize) -> Result<&mut Entry<T>> {
        self.entries
            .get_mut(index)
            .ok_or(SlotTrackerError::OutOfBounds)
    }

    pub fn get_common_value(&self, index: usize) -> Result<&T::Common> {
        Ok(&self.get_entry(index)?.common)
    }

    pub fn get_common_value_mut(&mut self, index: usize) -> Result<&mut T::Common> {
        Ok(&mut self.get_entry_mut(index)?.common)
    }

    pub fn get_state(&self, index: usize) -> Result<SlotState> {
        Ok(self.get_entry(index)?.state.project())
    }

    pub fn get_state_value(&self, index: usize) -> Result<SlotStateValueRef<T>> {
        Ok(match self.get_entry(index)?.state {
            StateInternal::Free { ref value, .. } => SlotStateValueRef::Free(value),
            StateInternal::Occupied { ref value, .. } => SlotStateValueRef::Occupied(value),
        })
    }

    pub fn get_state_value_mut(&mut self, index: usize) -> Result<SlotStateValueRefMut<T>> {
        Ok(match self.get_entry_mut(index)?.state {
            StateInternal::Free { ref mut value, .. } => SlotStateValueRefMut::Free(value),
            StateInternal::Occupied { ref mut value, .. } => SlotStateValueRefMut::Occupied(value),
        })
    }

    pub fn occupy(&mut self, value: T::Occupied) -> Option<(usize, T::Free)> {
        let index = self.free_list_head_index?;
        let new_state = StateInternal::Occupied { value };
        let old_state = self.replace_state(index, new_state);
        let value = match old_state {
            StateInternal::Free {
                free_list_next_index,
                value,
            } => {
                self.free_list_head_index = free_list_next_index;
                value
            }
            _ => {
                unreachable!()
            }
        };
        self.num_free -= 1;
        Some((index, value))
    }

    pub fn free(&mut self, index: usize, value: T::Free) -> Result<T::Occupied> {
        self.ensure_state(index, SlotState::Occupied)?;
        let new_state = StateInternal::Free {
            free_list_next_index: self.free_list_head_index.replace(index),
            value,
        };
        let old_state = self.replace_state(index, new_state);
        let value = match old_state {
            StateInternal::Occupied { value } => value,
            _ => unreachable!(),
        };
        self.num_free += 1;
        Ok(value)
    }

    fn ensure_state(&self, index: usize, state: SlotState) -> Result<()> {
        if self.get_state(index)? == state {
            Ok(())
        } else {
            Err(SlotTrackerError::StateMismatch)
        }
    }

    fn replace_state(&mut self, index: usize, new_state: StateInternal<T>) -> StateInternal<T> {
        mem::replace(&mut self.get_entry_mut(index).unwrap().state, new_state)
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SlotState {
    Free,
    Occupied,
}

impl SlotState {
    pub fn is_free(&self) -> bool {
        *self == Self::Free
    }

    pub fn is_occupied(&self) -> bool {
        *self == Self::Occupied
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SlotStateValueRef<'a, T: SlotStateTypes> {
    Free(&'a T::Free),
    Occupied(&'a T::Occupied),
}

impl<'a, T: SlotStateTypes> SlotStateValueRef<'a, T> {
    pub fn as_free(self) -> Result<&'a T::Free> {
        match self {
            Self::Free(r) => Ok(r),
            _ => Err(SlotTrackerError::StateMismatch),
        }
    }

    pub fn as_occupied(self) -> Result<&'a T::Occupied> {
        match self {
            Self::Occupied(r) => Ok(r),
            _ => Err(SlotTrackerError::StateMismatch),
        }
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SlotStateValueRefMut<'a, T: SlotStateTypes> {
    Free(&'a mut T::Free),
    Occupied(&'a mut T::Occupied),
}

impl<'a, T: SlotStateTypes> SlotStateValueRefMut<'a, T> {
    pub fn as_free(self) -> Result<&'a mut T::Free> {
        match self {
            Self::Free(r) => Ok(r),
            _ => Err(SlotTrackerError::StateMismatch),
        }
    }

    pub fn as_occupied(self) -> Result<&'a mut T::Occupied> {
        match self {
            Self::Occupied(r) => Ok(r),
            _ => Err(SlotTrackerError::StateMismatch),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SlotTrackerError {
    OutOfBounds,
    StateMismatch,
}
