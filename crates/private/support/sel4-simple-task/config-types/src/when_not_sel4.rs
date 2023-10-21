//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

macro_rules! dummies {
    {
        $($i:ident)*
    } => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $i;
        )*
    }
}

dummies! {
    CPtr

    Untyped
    Endpoint
    Notification
    TCB
    VCPU
    CNode
    SmallPage
    LargePage
    HugePage
    PGD
    PUD
    PD
    PT
    IRQControl
    IRQHandler
    ASIDControl
    ASIDPool
    Unspecified
    Null

    StaticThread
}
