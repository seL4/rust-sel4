//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::borrow::Borrow;
use std::rc::Rc;

// TODO mitigate regex size explosions with smart constructors

pub trait Predicate<T> {
    fn is_match(&self, c: &T) -> bool;
}

pub struct GenericRegex<P> {
    inner: Rc<Inner<P>>,
}

impl<P> Clone for GenericRegex<P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner().clone(),
        }
    }
}

impl<P> GenericRegex<P> {
    fn new(inner: Rc<Inner<P>>) -> Self {
        Self { inner }
    }

    fn inner(&self) -> &Rc<Inner<P>> {
        &self.inner
    }

    fn into_inner(self) -> Rc<Inner<P>> {
        self.inner
    }

    pub fn null() -> Self {
        Self::new(Inner::null())
    }

    pub fn epsilon() -> Self {
        Self::new(Inner::epsilon())
    }

    pub fn symbol(p: P) -> Self {
        Self::new(Inner::symbol(p))
    }

    pub fn complement(self) -> Self {
        Self::new(Inner::complement(self.into_inner()))
    }

    pub fn star(self) -> Self {
        Self::new(Inner::star(self.into_inner()))
    }

    pub fn or(self, rhs: Self) -> Self {
        Self::new(self.into_inner().or(rhs.into_inner()))
    }

    pub fn and(self, rhs: Self) -> Self {
        Self::new(self.into_inner().and(rhs.into_inner()))
    }

    pub fn then(self, rhs: Self) -> Self {
        Self::new(self.into_inner().then(rhs.into_inner()))
    }

    #[allow(dead_code)]
    pub fn traverse<Q, E>(
        &self,
        f: &mut impl FnMut(&P) -> Result<Q, E>,
    ) -> Result<GenericRegex<Q>, E> {
        self.inner().traverse(f).map(GenericRegex::new)
    }

    pub fn is_match<T>(&self, s: impl Iterator<Item = impl Borrow<T>>) -> bool
    where
        P: Predicate<T>,
    {
        let mut r = self.inner().clone();
        for x in s {
            r = r.derivative(x.borrow());
        }
        r.nullable()
    }

    // combinators

    pub fn plus(self) -> Self {
        Self::then(self.clone(), self.star())
    }

    pub fn optional(self) -> Self {
        Self::or(self, Self::epsilon())
    }

    pub fn repeat(self, min: Option<usize>, max: Option<usize>) -> Self {
        let lhs = match min {
            Some(n) => self.clone().repeat_at_least(n),
            None => Self::null().complement(),
        };
        let rhs = match max {
            Some(n) => self.clone().repeat_at_most(n),
            None => Self::null().complement(),
        };
        lhs.and(rhs)
    }

    pub fn repeat_exactly(self, n: usize) -> Self {
        match n {
            0 => Self::epsilon(),
            _ => Self::then(self.clone(), self.repeat_exactly(n - 1)),
        }
    }

    pub fn repeat_at_least(self, n: usize) -> Self {
        self.clone().repeat_exactly(n).then(self.star())
    }

    pub fn repeat_at_most(self, n: usize) -> Self {
        match n {
            0 => Self::epsilon(),
            _ => Self::or(
                Self::epsilon(),
                Self::then(self.clone(), self.repeat_exactly(n - 1)),
            ),
        }
    }
}

#[derive(Debug, Clone)]
enum Inner<P> {
    Null,
    Epsilon,
    Symbol(P),
    Complement(Rc<Self>),
    Star(Rc<Self>),
    Or(Rc<Self>, Rc<Self>),
    And(Rc<Self>, Rc<Self>),
    Then(Rc<Self>, Rc<Self>),
}

impl<P> Inner<P> {
    fn null() -> Rc<Self> {
        Rc::new(Self::Null)
    }

    fn epsilon() -> Rc<Self> {
        Rc::new(Self::Epsilon)
    }

    fn symbol(p: P) -> Rc<Self> {
        Rc::new(Self::Symbol(p))
    }

    fn complement(self: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Complement(self))
    }

    fn star(self: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Star(self))
    }

    fn or(self: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Or(self, rhs))
    }

    fn and(self: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::And(self, rhs))
    }

    fn then(self: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Then(self, rhs))
    }

    fn traverse<Q, E>(&self, f: &mut impl FnMut(&P) -> Result<Q, E>) -> Result<Rc<Inner<Q>>, E> {
        Ok(Rc::new(match self {
            Self::Null => Inner::Null,
            Self::Epsilon => Inner::Epsilon,
            Self::Symbol(p) => Inner::Symbol(f(p)?),
            Self::Complement(r) => Inner::Complement(r.traverse(f)?),
            Self::Star(r) => Inner::Star(r.traverse(f)?),
            Self::Or(lhs, rhs) => Inner::Or(lhs.traverse(f)?, rhs.traverse(f)?),
            Self::And(lhs, rhs) => Inner::And(lhs.traverse(f)?, rhs.traverse(f)?),
            Self::Then(lhs, rhs) => Inner::Then(lhs.traverse(f)?, rhs.traverse(f)?),
        }))
    }

    fn nullable(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Epsilon => true,
            Self::Symbol(_) => false,
            Self::Complement(r) => !r.nullable(),
            Self::Star(_) => true,
            Self::Or(lhs, rhs) => lhs.nullable() || rhs.nullable(),
            Self::And(lhs, rhs) => lhs.nullable() && rhs.nullable(),
            Self::Then(lhs, rhs) => lhs.nullable() && rhs.nullable(),
        }
    }

    fn derivative<T>(&self, c: &T) -> Rc<Self>
    where
        P: Predicate<T>,
    {
        match self {
            Self::Null => Self::null(),
            Self::Epsilon => Self::null(),
            Self::Symbol(p) => {
                if p.is_match(c) {
                    Self::epsilon()
                } else {
                    Self::null()
                }
            }
            Self::Complement(r) => Self::complement(r.derivative(c)),
            Self::Star(r) => r.derivative(c).then(r.clone().star()),
            Self::Or(lhs, rhs) => lhs.derivative(c).or(rhs.derivative(c)),
            Self::And(lhs, rhs) => lhs.derivative(c).and(rhs.derivative(c)),
            Self::Then(lhs, rhs) => lhs.derivative(c).then(rhs.clone()).or(if lhs.nullable() {
                rhs.derivative(c)
            } else {
                Self::null()
            }),
        }
    }
}
