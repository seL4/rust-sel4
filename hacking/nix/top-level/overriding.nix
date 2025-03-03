#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

self:

{

  overrideNixpkgsArgs = f: self.override (superArgs: selfBase:
    let
      concreteSuperArgs = superArgs selfBase;
    in
      concreteSuperArgs // {
        nixpkgsArgsForCrossSystem = crossSystem:
          f (concreteSuperArgs.nixpkgsArgsForCrossSystem crossSystem);
      }
  );

  withOverlays = overlays: self.overrideNixpkgsArgs (superNixpkgsArgs:
    superNixpkgsArgs // {
      overlays = superNixpkgsArgs.overlays ++ overlays;
    }
  );

  withConfigOverride = f: self.withOverlays [
    (self': super': {
      this = super'.this.overrideScope (self'': super'': {
        overridableScopeConfig = f super''.overridableScopeConfig;
      });
    })
  ];

  withClippy = self.withConfigOverride (attrs: attrs // {
    runClippyDefault = true;
  });

  withFerrocene = self.withConfigOverride (attrs: attrs // {
    rustEnvironmentSelector = (attrs.rustEnvironmentSelector or {}) // {
      tracks = "ferrocene";
      upstream = false;
    };
  });

  withVerus = self.withConfigOverride (attrs: attrs // {
    rustEnvironmentSelector = (attrs.rustEnvironmentSelector or {}) // {
      tracks = "verus";
    };
  });

  withUpstream = self.withConfigOverride (attrs: attrs // {
    rustEnvironmentSelector = (attrs.rustEnvironmentSelector or {}) // {
      upstream = true;
    };
  });

}
