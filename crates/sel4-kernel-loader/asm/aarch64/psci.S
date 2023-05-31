#include "macros.h"

BEGIN_FUNC(call_psci)
    smc #0
    ret
END_FUNC(call_psci)
