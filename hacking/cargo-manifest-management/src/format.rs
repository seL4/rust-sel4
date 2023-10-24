//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::mem;

use either::Either;
use serde_json::Value as JsonValue;
use toml_edit::{Array, ArrayOfTables, Document, Formatted, InlineTable, Item, Table, Value};

const MAX_WIDTH: usize = 100;

pub trait Policy {
    fn compare_keys(path: &[PathSegment], a: &str, b: &str) -> Ordering;

    fn is_always_table(path: &[PathSegment]) -> bool;

    fn is_always_array_of_tables(path: &[PathSegment]) -> bool;
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

pub fn format<P: Policy>(manifest: &JsonValue) -> Document {
    let mut state = ValueTranslation {
        current_path: vec![],
        _phantom: PhantomData::<P>,
    };
    let item = state.translate(manifest).into_item();
    let mut doc = Document::new();
    let _ = mem::replace(doc.as_item_mut(), item);
    doc
}

struct ValueTranslation<P> {
    current_path: Vec<PathSegment>,
    _phantom: PhantomData<P>,
}

impl<P: Policy> ValueTranslation<P> {
    fn current_key(&self) -> Option<&str> {
        self.current_path.last()?.as_table_key()
    }

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
                    self.current_path.push(PathSegment::ArrayIndex(i));
                    children.push(self.translate(x));
                    self.current_path.pop();
                }
                let homogenous_children = match TranslatedManyArray::homogenize(&children) {
                    TranslatedManyArray::Values(values)
                        if P::is_always_array_of_tables(&self.current_path) =>
                    {
                        TranslatedManyArray::Tables(TranslatedManyArray::tables_from_values(
                            &values,
                        ))
                    }
                    x => x,
                };
                match homogenous_children {
                    TranslatedManyArray::Values(values) => {
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
                        Translated::Value(Value::Array(array))
                    }
                    TranslatedManyArray::Tables(tables) => {
                        let mut array = ArrayOfTables::new();
                        for v in tables.into_iter() {
                            array.push(v);
                        }
                        Translated::Item(Item::ArrayOfTables(array))
                    }
                }
            }
            JsonValue::Object(w) => {
                let mut children = BTreeMap::new();
                for (k, x) in w.iter() {
                    self.current_path.push(PathSegment::TableKey(k.clone()));
                    children.insert(k.clone(), self.translate(x));
                    self.current_path.pop();
                }
                let homogenous_children = match TranslatedManyMap::homogenize(&children) {
                    TranslatedManyMap::Values(values) if P::is_always_table(&self.current_path) => {
                        TranslatedManyMap::Items(TranslatedManyMap::items_from_values(values))
                    }
                    x => x,
                };
                let inline_table_or_items = match homogenous_children {
                    TranslatedManyMap::Values(values) => {
                        let mut table = InlineTable::new();
                        for (k, v) in values.clone().into_iter() {
                            table.insert(k, v);
                        }
                        table.sort_values_by(|k1, _v1, k2, _v2| {
                            P::compare_keys(&self.current_path, k1, k2)
                        });
                        if !self
                            .current_key()
                            .map(|k| is_kv_too_long(k, &Value::InlineTable(table.clone())))
                            .unwrap_or(false)
                        {
                            Either::Left(table)
                        } else {
                            Either::Right(TranslatedManyMap::items_from_values(values))
                        }
                    }
                    TranslatedManyMap::Items(items) => Either::Right(items),
                };
                match inline_table_or_items {
                    Either::Left(inline_table) => {
                        Translated::Value(Value::InlineTable(inline_table))
                    }
                    Either::Right(items) => {
                        let mut table = Table::new();
                        table.set_implicit(true);
                        for (k, v) in items.into_iter() {
                            table.insert(&k, v);
                        }
                        table.sort_values_by(|k1, _v1, k2, _v2| {
                            P::compare_keys(&self.current_path, k1, k2)
                        });
                        Translated::Item(Item::Table(table))
                    }
                }
            }
            _ => panic!(),
        }
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

enum TranslatedManyMap {
    Items(BTreeMap<String, Item>),
    Values(BTreeMap<String, Value>),
}

impl TranslatedManyMap {
    fn homogenize(heterogeneous: &BTreeMap<String, Translated>) -> Self {
        if heterogeneous.values().all(Translated::is_value) {
            TranslatedManyMap::Values(
                heterogeneous
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_value().unwrap().clone()))
                    .collect(),
            )
        } else {
            TranslatedManyMap::Items(
                heterogeneous
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone().into_item()))
                    .collect(),
            )
        }
    }

    fn items_from_values(values: BTreeMap<String, Value>) -> BTreeMap<String, Item> {
        values
            .into_iter()
            .map(|(k, v)| (k, Item::Value(v)))
            .collect()
    }
}

enum TranslatedManyArray {
    Tables(Vec<Table>),
    Values(Vec<Value>),
}

impl TranslatedManyArray {
    fn homogenize(heterogeneous: &[Translated]) -> Self {
        if heterogeneous.iter().all(Translated::is_value) {
            TranslatedManyArray::Values(
                heterogeneous
                    .iter()
                    .map(|v| v.as_value().unwrap().clone())
                    .collect(),
            )
        } else {
            TranslatedManyArray::Tables(
                heterogeneous
                    .iter()
                    .map(|v| v.clone().into_item().as_table().unwrap().clone())
                    .collect(),
            )
        }
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
