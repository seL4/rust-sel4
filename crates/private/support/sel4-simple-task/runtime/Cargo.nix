#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, serdeWith, postcardWith, versions }:

mk {
  package.name = "sel4-simple-task-runtime";
  dependencies = {
    serde = serdeWith [];
    postcard = postcardWith [];
    serde_json = { version = versions.serde_json; default-features = false; optional = true; };

    inherit (localCrates)
      sel4
      sel4-dlmalloc
      sel4-immediate-sync-once-cell
      sel4-panicking
      sel4-panicking-env
      sel4-simple-task-runtime-config-types
      sel4-simple-task-runtime-macros
      sel4-simple-task-threading
      sel4-sync
      sel4-ctors-dtors
    ;
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "tls" ]; };
  };
  target."cfg(not(target_arch = \"arm\"))".dependencies = {
    sel4-backtrace = localCrates.sel4-backtrace // { features = [ "unwinding" "postcard" ]; };
    inherit (localCrates) sel4-backtrace-simple;
  };
  features = {
    serde_json = [
      "dep:serde_json"
    ];
    alloc = [
      "sel4-backtrace/alloc"
      "sel4-backtrace-simple/alloc"
      "sel4-simple-task-threading/alloc"
      "serde_json?/alloc"
    ];
    default = [
      "serde_json"
      "alloc"
    ];
  };
}
