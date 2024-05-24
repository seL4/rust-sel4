//
// Copyright 2024, Colias Group, LLC
// Copyright (c) Microsoft Corporation
//
// SPDX-License-Identifier: MIT
//

method Max(a: int, b:int) returns (c: int)
  ensures a < b  ==> c == b
  ensures b <= a ==> c == a
{
  if (a < b) {
    return b;
  } else {
    return a;
  }
}
