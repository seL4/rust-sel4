//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub type RawConfigWord = u64;

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
    Tcb
    VCpu
    CNode
    SmallPage
    LargePage
    HugePage
    PageGlobalDirectory
    PageUpperDirectory
    PageDirectory
    PageTable
    IrqControl
    IrqHandler
    AsidControl
    AsidPool
    Unspecified
    Null

    StaticThread
}
