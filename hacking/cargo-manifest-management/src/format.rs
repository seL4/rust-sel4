//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::mem;

use either::Either;
use serde_json::Value as JsonValue;
use toml_edit::{Array, ArrayOfTables, Document, Formatted, InlineTable, Item, Table, Value};

const MAX_WIDTH: usize = 100;

pub trait Policy {
    fn compare_keys(&self, path: &[PathSegment], a: &str, b: &str) -> Ordering;

    fn is_always_table(&self, path: &[PathSegment]) -> bool;

    fn is_always_array_of_tables(&self, path: &[PathSegment]) -> bool;
}

pub enum PathSegment {
    TableKey(String),
    ArrayIndex(usize),
}

impl PathSegment {
    pub fn as_table_key(&self) -> Option<&str> {
        match self {
            Self::TableKey(k) => Some(k),
            _ => None,
        }
    }

    pub fn is_table_key(&self, key: &str) -> bool {
        self.as_table_key().map(|k| k == key).unwrap_or(false)
    }
}

pub struct Formatter<P> {
    policy: P,
}

impl<P: Policy> Formatter<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    pub fn format(&self, manifest: &JsonValue) -> Document {
        let mut state = FormatterState {
            formatter: self,
            current_path: vec![],
        };
        let item = state.translate(manifest).into_item();
        let mut doc = Document::new();
        let _ = mem::replace(doc.as_item_mut(), item);
        doc
    }
}

struct FormatterState<'a, P> {
    formatter: &'a Formatter<P>,
    current_path: Vec<PathSegment>,
}

impl<'a, P: Policy> FormatterState<'a, P> {
    fn translate(&mut self, v: &JsonValue) -> Translated {
        match v {
            JsonValue::Bool(w) => Value::Boolean(Formatted::new(*w)).into(),
            JsonValue::Number(w) => if let Some(x) = w.as_i64() {
                Value::Integer(Formatted::new(x))
            } else if let Some(x) = w.as_f64() {
                Value::Float(Formatted::new(x))
            } else {
                panic!()
            }
            .into(),
            JsonValue::String(w) => Value::String(Formatted::new(w.clone())).into(),
            JsonValue::Array(w) => {
                let mut children = vec![];
                for (i, x) in w.iter().enumerate() {
                    self.push(PathSegment::ArrayIndex(i));
                    children.push(self.translate(x));
                    self.pop();
                }
                let homogenous_children =
                    if self.policy().is_always_array_of_tables(self.current_path()) {
                        TranslatedArray::Tables(TranslatedArray::tables_from_translated(&children))
                    } else {
                        TranslatedArray::homogenize(&children)
                    };
                match homogenous_children {
                    TranslatedArray::Values(values) => Translated::Value(Value::Array({
                        let mut array = Array::new();
                        for v in values.into_iter() {
                            array.push(v);
                        }
                        let wrap = self
                            .current_key()
                            .map(|k| is_kv_too_long(k, &Value::Array(array.clone())))
                            .unwrap_or(false);
                        if wrap {
                            array.iter_mut().for_each(|e| {
                                e.decor_mut().set_prefix("\n    ");
                            });
                            array.iter_mut().last().map(|e| {
                                e.decor_mut().set_suffix("\n");
                            });
                        }
                        array
                    })),
                    TranslatedArray::Tables(tables) => Translated::Item(Item::ArrayOfTables({
                        let mut array = ArrayOfTables::new();
                        for v in tables.into_iter() {
                            array.push(v);
                        }
                        array
                    })),
                }
            }
            JsonValue::Object(w) => {
                let mut children = BTreeMap::new();
                for (k, x) in w.iter() {
                    self.push(PathSegment::TableKey(k.clone()));
                    children.insert(k.clone(), self.translate(x));
                    self.pop();
                }
                let homogenous_children = if self.policy().is_always_table(self.current_path()) {
                    TranslatedMap::Items(TranslatedMap::items_from_translated(&children))
                } else {
                    TranslatedMap::homogenize(&children)
                };
                match homogenous_children {
                    TranslatedMap::Values(values) => {
                        let mut table = InlineTable::new();
                        for (k, v) in values.clone().into_iter() {
                            table.insert(k, v);
                        }
                        table.sort_values_by(|k1, _v1, k2, _v2| {
                            self.policy().compare_keys(self.current_path(), k1, k2)
                        });
                        if !self
                            .current_key()
                            .map(|k| is_kv_too_long(k, &Value::InlineTable(table.clone())))
                            .unwrap_or(false)
                        {
                            Either::Left(Translated::Value(Value::InlineTable(table)))
                        } else {
                            Either::Right(TranslatedMap::items_from_values(values))
                        }
                    }
                    TranslatedMap::Items(items) => Either::Right(items),
                }
                .map_right(|items| {
                    let mut table = Table::new();
                    table.set_implicit(true);
                    for (k, v) in items.into_iter() {
                        table.insert(&k, v);
                    }
                    table.sort_values_by(|k1, _v1, k2, _v2| {
                        self.policy().compare_keys(self.current_path(), k1, k2)
                    });
                    Translated::Item(Item::Table(table))
                })
                .into_inner()
            }
            _ => panic!(),
        }
    }

    fn policy(&self) -> &P {
        &self.formatter.policy
    }

    fn current_path(&self) -> &[PathSegment] {
        &self.current_path
    }

    fn current_key(&self) -> Option<&str> {
        self.current_path.last()?.as_table_key()
    }
    fn push(&mut self, seg: PathSegment) {
        self.current_path.push(seg);
    }

    fn pop(&mut self) {
        self.current_path.pop();
    }
}

#[derive(Clone)]
enum Translated {
    Item(Item),
    Value(Value),
}

impl From<Item> for Translated {
    fn from(item: Item) -> Self {
        Self::Item(item)
    }
}

impl From<Value> for Translated {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl Translated {
    fn as_value(&self) -> Option<&Value> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }

    fn is_value(&self) -> bool {
        self.as_value().is_some()
    }

    fn into_item(self) -> Item {
        match self {
            Self::Value(value) => Item::Value(value),
            Self::Item(item) => item,
        }
    }
}

enum TranslatedMap {
    Items(BTreeMap<String, Item>),
    Values(BTreeMap<String, Value>),
}

impl TranslatedMap {
    fn homogenize(heterogeneous: &BTreeMap<String, Translated>) -> Self {
        if heterogeneous.values().all(Translated::is_value) {
            Self::Values(
                heterogeneous
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_value().unwrap().clone()))
                    .collect(),
            )
        } else {
            Self::Items(Self::items_from_translated(heterogeneous))
        }
    }

    fn items_from_translated(translated: &BTreeMap<String, Translated>) -> BTreeMap<String, Item> {
        translated
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into_item()))
            .collect()
    }

    fn items_from_values(values: BTreeMap<String, Value>) -> BTreeMap<String, Item> {
        values
            .into_iter()
            .map(|(k, v)| (k, Item::Value(v)))
            .collect()
    }
}

enum TranslatedArray {
    Tables(Vec<Table>),
    Values(Vec<Value>),
}

impl TranslatedArray {
    fn homogenize(heterogeneous: &[Translated]) -> Self {
        if heterogeneous.iter().all(Translated::is_value) {
            Self::Values(
                heterogeneous
                    .iter()
                    .map(|v| v.as_value().unwrap().clone())
                    .collect(),
            )
        } else {
            Self::Tables(Self::tables_from_translated(heterogeneous))
        }
    }

    fn tables_from_translated(translated: &[Translated]) -> Vec<Table> {
        translated
            .iter()
            .map(|v| v.clone().into_item().as_table().unwrap().clone())
            .collect()
    }

    fn tables_from_values(values: &[Value]) -> Vec<Table> {
        values
            .into_iter()
            .map(|v| v.as_inline_table().unwrap().clone().into_table())
            .collect()
    }
}

fn is_kv_too_long(k: &str, v: &Value) -> bool {
    let mut table = Table::new();
    table.insert(k, Item::Value(v.clone()));
    let mut doc = Document::new();
    let _ = mem::replace(doc.as_item_mut(), Item::Table(table));
    let mut s = doc.to_string();
    assert_eq!(s.pop(), Some('\n'));
    s.chars().count() > MAX_WIDTH
}
