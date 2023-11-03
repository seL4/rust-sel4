//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::Ordering;
use std::mem;

use toml::value::{
    Array as UnformattedArray, Table as UnformattedTable, Value as UnformattedValue,
};
use toml_edit::{Array, ArrayOfTables, Document, Formatted, InlineTable, Item, Key, Table, Value};

use toml_path_regex::{Path, PathSegment};

pub trait Policy {
    fn max_width(&self) -> usize;

    fn indent_width(&self) -> usize;

    fn never_inline_table(&self, path: &[PathSegment]) -> bool;

    fn compare_keys(&self, path: &[PathSegment], a: &str, b: &str) -> Ordering;
}

pub struct Formatter<P> {
    policy: P,
}

impl<P: Policy> Formatter<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    pub fn format(&self, table: &UnformattedTable) -> Result<Document, Error> {
        let mut state = FormatterState {
            formatter: self,
            current_path: Path::new(),
        };
        state.format_top_level(table)
    }
}

struct FormatterState<'a, P> {
    formatter: &'a Formatter<P>,
    current_path: Path,
}

#[derive(Debug, Clone)]
pub enum Error {
    CannotInlineTable(CannotInlineTableError),
}

#[derive(Debug, Clone)]
pub struct CannotInlineTableError {
    path: Path,
}

impl CannotInlineTableError {
    fn new(path: Path) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &[PathSegment] {
        self.path.as_slice()
    }
}

impl From<CannotInlineTableError> for Error {
    fn from(err: CannotInlineTableError) -> Error {
        Self::CannotInlineTable(err)
    }
}

impl<'a, P: Policy> FormatterState<'a, P> {
    fn format_top_level(&mut self, v: &UnformattedTable) -> Result<Document, Error> {
        let table = self.format_table_to_table(v)?;
        let mut doc = Document::new();
        *doc.as_table_mut() = table;
        Ok(doc)
    }

    fn format_to_value(&mut self, v: &UnformattedValue) -> Result<Value, Error> {
        Ok(match v {
            UnformattedValue::Boolean(w) => Value::Boolean(Formatted::new(*w)),
            UnformattedValue::Integer(w) => Value::Integer(Formatted::new(*w)),
            UnformattedValue::Float(w) => Value::Float(Formatted::new(*w)),
            UnformattedValue::String(w) => Value::String(Formatted::new(w.clone())),
            UnformattedValue::Datetime(w) => Value::Datetime(Formatted::new(w.clone())),
            UnformattedValue::Array(w) => Value::Array(self.format_array_to_array(w)?),
            UnformattedValue::Table(w) => Value::InlineTable(self.format_table_to_inline_table(w)?),
        })
    }

    fn format_table_to_inline_table(&mut self, v: &UnformattedTable) -> Result<InlineTable, Error> {
        if self.policy().never_inline_table(self.current_path()) {
            return Err(CannotInlineTableError::new(self.current_path.clone()).into());
        }
        let mut table = InlineTable::new();
        for (k, x) in v.iter() {
            table.insert(k, self.with_key(k, |this| this.format_to_value(x))?);
        }
        table.sort_values_by(self.compare_values_at_current_path());
        Ok(table)
    }

    fn format_array_to_array(&mut self, v: &UnformattedArray) -> Result<Array, Error> {
        let mut array = Array::new();
        for (i, x) in v.iter().enumerate() {
            array.push(self.with_index(i, |this| this.format_to_value(x))?);
        }
        Ok(array)
    }

    fn format_table_to_table(&mut self, v: &UnformattedTable) -> Result<Table, Error> {
        let mut table = Table::new();
        table.set_implicit(true);
        for (k, x) in v.iter() {
            table.insert(k, self.with_key(k, |this| this.format_to_item(x))?);
        }
        table.sort_values_by(self.compare_values_at_current_path());
        Ok(table)
    }

    fn format_to_item(&mut self, v: &UnformattedValue) -> Result<Item, Error> {
        Ok(match v {
            UnformattedValue::Array(w) => self.format_array_to_item(w)?,
            UnformattedValue::Table(w) => self.format_table_to_item(w)?,
            _ => Item::Value(self.format_to_value(v)?),
        })
    }

    fn format_array_to_item(&mut self, v: &UnformattedArray) -> Result<Item, Error> {
        let just_tables = v.iter().all(UnformattedValue::is_table);
        let mut array = if !just_tables {
            Some(self.format_array_to_array(v)?)
        } else {
            let mut inline_tables = Some(vec![]);
            for (i, x) in v
                .iter()
                .map(UnformattedValue::as_table)
                .map(Option::unwrap)
                .enumerate()
            {
                let inline_table =
                    self.with_index(i, |this| this.format_table_to_inline_table(x))?;
                if self.is_inline_table_too_wide_for_array(&inline_table) {
                    inline_tables = None;
                    break;
                } else {
                    inline_tables.as_mut().unwrap().push(inline_table);
                }
            }
            inline_tables.map(|inline_tables| {
                let mut array = Array::new();
                array.extend(inline_tables);
                array
            })
        };
        if let Some(array) = &mut array {
            let wrap =
                self.is_kv_too_wide(self.current_key().unwrap(), &Value::Array(array.clone()));
            if wrap {
                array.iter_mut().for_each(|e| {
                    e.decor_mut().set_prefix("\n    ");
                });
                array.set_trailing_comma(true);
                array.set_trailing("\n");
            }
        }
        Ok(if let Some(array) = array {
            Item::Value(Value::Array(array))
        } else {
            let mut array_of_tables = ArrayOfTables::new();
            for (i, x) in v
                .iter()
                .map(UnformattedValue::as_table)
                .map(Option::unwrap)
                .enumerate()
            {
                let table = self.with_index(i, |this| this.format_table_to_table(x))?;
                array_of_tables.push(table);
            }
            Item::ArrayOfTables(array_of_tables)
        })
    }

    fn format_table_to_item(&mut self, v: &UnformattedTable) -> Result<Item, Error> {
        let inline_table = match self.format_table_to_inline_table(v) {
            Ok(inline_table) => {
                let too_wide = self.is_kv_too_wide(
                    self.current_key().unwrap(),
                    &Value::InlineTable(inline_table.clone()),
                );
                if !too_wide {
                    Some(inline_table)
                } else {
                    None
                }
            }
            Err(Error::CannotInlineTable(_)) => None,
        };
        Ok(if let Some(inline_table) = inline_table {
            Item::Value(Value::InlineTable(inline_table))
        } else {
            let table = self.format_table_to_table(v)?;
            Item::Table(table)
        })
    }

    fn policy(&self) -> &P {
        &self.formatter.policy
    }

    fn current_path(&self) -> &[PathSegment] {
        self.current_path.as_slice()
    }

    fn current_key(&self) -> Option<&str> {
        self.current_path().last()?.as_key()
    }
    fn push(&mut self, seg: PathSegment) {
        self.current_path.push(seg);
    }

    fn pop(&mut self) {
        self.current_path.pop();
    }

    fn with_path_segment<R>(&mut self, seg: PathSegment, f: impl FnOnce(&mut Self) -> R) -> R {
        self.push(seg);
        let r = f(self);
        self.pop();
        r
    }

    fn with_key<R>(&mut self, k: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_path_segment(PathSegment::Key(k.to_owned()), f)
    }

    fn with_index<R>(&mut self, i: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_path_segment(PathSegment::Index(i), f)
    }

    fn compare_values_at_current_path<T>(
        &mut self,
    ) -> impl '_ + FnMut(&Key, &T, &Key, &T) -> Ordering {
        |k1, _v1, k2, _v2| self.policy().compare_keys(self.current_path(), k1, k2)
    }

    fn is_kv_too_wide(&self, k: &str, v: &Value) -> bool {
        let mut table = Table::new();
        table.insert(k, Item::Value(v.clone()));
        let mut doc = Document::new();
        let _ = mem::replace(doc.as_item_mut(), Item::Table(table));
        let mut s = doc.to_string();
        assert_eq!(s.pop(), Some('\n'));
        s.chars().count() > self.policy().max_width()
    }

    fn is_inline_table_too_wide_for_array(&self, v: &InlineTable) -> bool {
        let value = format!("{}", v).len();
        let comma = 1;
        let total = self.policy().indent_width() + value + comma;
        total > self.policy().max_width()
    }
}
