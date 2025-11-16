SECTIONS {
    .persistent : {
        *(.persistent .persistent.*)
    }
} INSERT BEFORE .data;

ASSERT(DEFINED(_reset), "_reset is not defined")

ENTRY(_reset)
