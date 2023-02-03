use crate::{object, NamedObject, Object};

impl<N, C, F> NamedObject<N, C, F> {
    pub fn traverse_simple<N1, C1, F1, E>(
        &self,
        f: impl FnOnce(&N) -> Result<N1, E>,
        g: impl FnOnce(&C) -> Result<C1, E>,
        h: impl FnOnce(&F) -> Result<F1, E>,
    ) -> Result<NamedObject<N1, C1, F1>, E> {
        Ok(NamedObject {
            name: f(&self.name)?,
            object: self.object.traverse_simple(g, h)?,
        })
    }
}

impl<C, F> Object<C, F> {
    pub fn traverse_simple<C1, F1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
        g: impl FnOnce(&F) -> Result<F1, E>,
    ) -> Result<Object<C1, F1>, E> {
        Ok(match self {
            Object::Untyped(obj) => Object::Untyped(obj.clone()),
            Object::Endpoint => Object::Endpoint,
            Object::Notification => Object::Notification,
            Object::CNode(obj) => Object::CNode(obj.traverse_simple(f)?),
            Object::TCB(obj) => Object::TCB(obj.traverse_simple(f)?),
            Object::IRQ(obj) => Object::IRQ(obj.traverse_simple(f)?),
            Object::VCPU => Object::VCPU,
            Object::SmallPage(obj) => Object::SmallPage(obj.traverse_simple(g)?),
            Object::LargePage(obj) => Object::LargePage(obj.traverse_simple(g)?),
            Object::PT(obj) => Object::PT(obj.traverse_simple(f)?),
            Object::PD(obj) => Object::PD(obj.traverse_simple(f)?),
            Object::PUD(obj) => Object::PUD(obj.traverse_simple(f)?),
            Object::PGD(obj) => Object::PGD(obj.traverse_simple(f)?),
            Object::ASIDPool(obj) => Object::ASIDPool(obj.clone()),
            Object::ArmIRQ(obj) => Object::ArmIRQ(obj.traverse_simple(f)?),
        })
    }
}

impl<F> object::SmallPage<F> {
    pub fn traverse_simple<F1, E>(
        &self,
        f: impl FnOnce(&F) -> Result<F1, E>,
    ) -> Result<object::SmallPage<F1>, E> {
        Ok(object::SmallPage {
            paddr: self.paddr,
            fill: f(&self.fill)?,
        })
    }
}

impl<F> object::LargePage<F> {
    pub fn traverse_simple<F1, E>(
        &self,
        f: impl FnOnce(&F) -> Result<F1, E>,
    ) -> Result<object::LargePage<F1>, E> {
        Ok(object::LargePage {
            paddr: self.paddr,
            fill: f(&self.fill)?,
        })
    }
}

impl<C> object::CNode<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::CNode<C1>, E> {
        Ok(object::CNode {
            size_bits: self.size_bits,
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::TCB<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::TCB<C1>, E> {
        Ok(object::TCB {
            slots: f(&self.slots)?,
            fault_ep: self.fault_ep,
            extra_info: self.extra_info.clone(),
            init_args: self.init_args.clone(),
        })
    }
}

impl<C> object::IRQ<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::IRQ<C1>, E> {
        Ok(object::IRQ {
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::PT<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::PT<C1>, E> {
        Ok(object::PT {
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::PD<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::PD<C1>, E> {
        Ok(object::PD {
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::PUD<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::PUD<C1>, E> {
        Ok(object::PUD {
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::PGD<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::PGD<C1>, E> {
        Ok(object::PGD {
            slots: f(&self.slots)?,
        })
    }
}

impl<C> object::ArmIRQ<C> {
    pub fn traverse_simple<C1, E>(
        &self,
        f: impl FnOnce(&C) -> Result<C1, E>,
    ) -> Result<object::ArmIRQ<C1>, E> {
        Ok(object::ArmIRQ {
            slots: f(&self.slots)?,
            trigger: self.trigger,
            target: self.target,
        })
    }
}
