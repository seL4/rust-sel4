use std::env;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub const SEL4_PREFIX_ENV: &str = "SEL4_PREFIX";

pub const SEL4_PLATFORM_INFO: Var<PathVarType<'static>> = Var::new(
    "SEL4_PLATFORM_INFO",
    SEL4_PREFIX_ENV,
    "support/platform_gen.yaml",
);

pub const SEL4_INCLUDE_DIRS: Var<PathsVarType<'static>> = Var::new(
    "SEL4_INCLUDE_DIRS",
    SEL4_PREFIX_ENV,
    ["libsel4/include"].as_slice(),
);

pub struct Var<'a, T: VarType> {
    var: &'a str,
    default_prefix_var: &'a str,
    default_suffix: T::Suffix,
    phantom: PhantomData<T>,
}

impl<'a, T: VarType> Var<'a, T> {
    pub const fn new(var: &'a str, default_prefix_var: &'a str, default_suffix: T::Suffix) -> Self {
        Self {
            var,
            default_prefix_var,
            default_suffix,
            phantom: PhantomData,
        }
    }

    fn try_get_default(&self) -> Option<T::Value> {
        env::var(self.default_prefix_var)
            .ok()
            .map(PathBuf::from)
            .map(|prefix| T::from_suffix(&prefix, &self.default_suffix))
    }

    pub fn try_get(&self) -> Option<T::Value> {
        self.declare_as_dependency();
        env::var(self.var)
            .ok()
            .map(|raw_value| T::from_raw_value(&raw_value))
            .or_else(|| self.try_get_default())
    }

    pub fn get(&self) -> T::Value {
        self.try_get()
            .unwrap_or_else(|| panic!("{} or {} must be set", &self.var, &self.default_prefix_var))
    }

    pub fn declare_as_dependency(&self) {
        println!("cargo:rerun-if-env-changed={}", &self.var);
        println!("cargo:rerun-if-env-changed={}", &self.default_prefix_var);
    }
}

pub trait VarType {
    type Value;
    type Suffix;

    fn from_suffix(prefix: &Path, suffix: &Self::Suffix) -> Self::Value;
    fn from_raw_value(raw_value: &str) -> Self::Value;
}

pub struct SimpleVar<'a, T: VarType> {
    var: &'a str,
    phantom: PhantomData<T>,
}

impl<'a, T: VarType> SimpleVar<'a, T> {
    pub const fn new(var: &'a str) -> Self {
        Self {
            var,
            phantom: PhantomData,
        }
    }

    pub fn try_get(&self) -> Option<T::Value> {
        self.declare_as_dependency();
        env::var(self.var)
            .ok()
            .map(|raw_value| T::from_raw_value(&raw_value))
    }

    pub fn get(&self) -> T::Value {
        self.try_get()
            .unwrap_or_else(|| panic!("{} must be set", &self.var))
    }

    pub fn declare_as_dependency(&self) {
        println!("cargo:rerun-if-env-changed={}", &self.var);
    }
}

pub struct PathVarType<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> VarType for PathVarType<'a> {
    type Value = PathBuf;
    type Suffix = &'a str;

    fn from_suffix(prefix: &Path, suffix: &Self::Suffix) -> Self::Value {
        prefix.join(suffix)
    }

    fn from_raw_value(raw_value: &str) -> Self::Value {
        raw_value.into()
    }
}

pub struct PathsVarType<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> VarType for PathsVarType<'a> {
    type Value = Vec<PathBuf>;
    type Suffix = &'a [&'a str];

    fn from_suffix(prefix: &Path, suffix: &Self::Suffix) -> Self::Value {
        suffix.iter().map(|suffix| prefix.join(suffix)).collect()
    }

    fn from_raw_value(raw_value: &str) -> Self::Value {
        raw_value.split(':').map(PathBuf::from).collect()
    }
}

// // //

pub fn observe_path<T: AsRef<Path>>(path: T) -> T {
    println!("cargo:rerun-if-changed={}", path.as_ref().display());
    path
}
