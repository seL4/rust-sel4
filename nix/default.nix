let
  nixpkgsFn = import (builtins.getFlake "nixpkgs/227de2b3bbec142f912c09d5e8a1b4e778aa54fb").outPath;

  mkPkgs = crossSystem: nixpkgsFn {
    inherit crossSystem;
    overlays = [
      (self: super: {
        pythonPackagesExtensions = super.pythonPackagesExtensions ++ [
          (self.callPackage ./python-overrides.nix {})
        ];
      })
    ];
  };

  pkgs = {
    dev = mkPkgs null;
    aarch64 = mkPkgs { config = "aarch64-none-elf"; };
    riscv64 = mkPkgs { config = "riscv64-none-elf"; };
  };

  inherit (pkgs.dev) lib;

  configHelpers = pkgs.dev.callPackage ./config-helpers.nix {};

  mkWorld =
    { crossPkgs, kernelConfig, loaderConfig
    , rustSeL4Target, rustBareMetalTarget
    } @ args:
    let
    in args // rec {
      kernel = crossPkgs.callPackage ./kernel.nix {} {
        config = kernelConfig;
      };
      shell = crossPkgs.callPackage ./shell.nix {} {
        inherit kernel loaderConfig;
        inherit rustSeL4Target rustBareMetalTarget;
      };
    };

  worlds =
    with configHelpers;
    let
      loaderConfig = {};
    in {
      aarch64 =
        let
          crossPkgs = pkgs.aarch64;
          rustSeL4Target = "aarch64-unknown-sel4";
          rustBareMetalTarget = "aarch64-unknown-none";
        in {
          default = mkWorld {
            inherit crossPkgs loaderConfig;
            inherit rustSeL4Target rustBareMetalTarget;
            kernelConfig = {
              ARM_CPU = mkString "cortex-a57";
              KernelArch = mkString "arm";
              KernelSel4Arch = mkString "aarch64";
              KernelPlatform = mkString "qemu-arm-virt";
              KernelArmHypervisorSupport = on;
              KernelMaxNumNodes = mkString "2";
              KernelVerificationBuild = off;
            };
          };
        };
      riscv64 =
        let
          crossPkgs = pkgs.riscv64;
          rustSeL4Target = "riscv64imac-unknown-none-elf";
          rustBareMetalTarget = "riscv64imac-unknown-none-elf";
        in {
          default = mkWorld {
            inherit crossPkgs loaderConfig;
            inherit rustSeL4Target rustBareMetalTarget;
            kernelConfig = {
              KernelArch = mkString "riscv";
              KernelSel4Arch = mkString "riscv64";
              KernelPlatform = mkString "spike";
              KernelVerificationBuild = off;
            };
          };
        };
    };

in {
  inherit pkgs worlds;
}
