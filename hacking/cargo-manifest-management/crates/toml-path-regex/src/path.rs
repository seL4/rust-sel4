//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fmt;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

impl PathSegment {
    pub fn as_key(&self) -> Option<&str> {
        match self {
            Self::Key(k) => Some(k),
            _ => None,
        }
    }

    pub fn is_key(&self) -> bool {
        self.as_key().is_some()
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_index(&self) -> bool {
        self.as_index().is_some()
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "{:?}", key),
            Self::Index(index) => write!(f, "{:?}", index),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Path {
    inner: Vec<PathSegment>,
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[PathSegment] {
        self.inner.as_slice()
    }

    pub fn push(&mut self, path_segment: PathSegment) {
        self.inner.push(path_segment)
    }

    pub fn push_key(&mut self, key: String) {
        self.push(PathSegment::Key(key))
    }

    pub fn push_index(&mut self, index: usize) {
        self.push(PathSegment::Index(index))
    }

    pub fn pop(&mut self) -> Option<PathSegment> {
        self.inner.pop()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for seg in self.as_slice().iter() {
            write!(f, "[{}]", seg)?;
        }
        Ok(())
    }
}
