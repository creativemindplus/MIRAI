// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::bool_domain::BoolDomain;

use log_derive::logfn_inputs;
use mirai_annotations::*;
use rpds::{rbt_map, RedBlackTreeMap};
use rustc_hir::def_id::DefId;

/// Check if a value of enum type `TagPropagation` is included in a tag propagation set.
macro_rules! does_propagate_tag {
    ($set:expr, $x:expr) => {
        ($set & (1 << ($x as u8))) != 0
    };
}

/// A tag is represented as a pair of its tag kind and its propagation set.
/// The tag kind is the name of the declared tag type, and the propagation set is associated to the
/// tag type as a const generic parameter.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Tag {
    pub def_id: DefId,
    pub prop_set: TagPropagationSet,
}

/// An element of the tag domain is essentially an over-approximation for present and absent tags.
/// The approximation is denoted as a map from tags to lifted Boolean values (`BoolDomain`).
/// If a tag is mapped to `True`, then it must be present.
/// If a tag is mapped to `False', then it must be absent.
/// If a tag is mapped to `Top`, then it may or may not be present.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
pub struct TagDomain {
    map: RedBlackTreeMap<Tag, BoolDomain>,
}

/// Constructors
impl TagDomain {
    /// Construct a tag domain element representing an empty set.
    #[logfn_inputs(TRACE)]
    pub fn for_empty_set() -> TagDomain {
        TagDomain { map: rbt_map![] }
    }
}

/// Transfer functions
impl TagDomain {
    /// Return a new tag domain element by adding the `tag` whose presence is indicated by `val`.
    #[logfn_inputs(TRACE)]
    pub fn add_tag(&self, tag: Tag, val: BoolDomain) -> Self {
        TagDomain {
            map: self.map.insert(tag, val),
        }
    }

    /// Return a lifted Boolean that indicates the presence of `tag` in the tag domain element.
    #[logfn_inputs(TRACE)]
    pub fn has_tag(&self, tag: &Tag) -> BoolDomain {
        *self.map.get(tag).unwrap_or(&BoolDomain::False)
    }

    /// Return the union of two tag domain elements, which is pointwise logical-or on lifted Booleans.
    #[logfn_inputs(TRACE)]
    pub fn union(&self, other: &Self) -> Self {
        let mut new_map = rbt_map![];
        for (tag, val) in self.map.iter().chain(other.map.iter()) {
            let cur_val = *new_map.get(tag).unwrap_or(&BoolDomain::False);
            let new_val = cur_val.or(val);
            new_map.insert_mut(*tag, new_val);
        }
        TagDomain { map: new_map }
    }

    /// Return the join of two tag domain elements, which is pointwise join on lifted Booleans.
    #[logfn_inputs(TRACE)]
    pub fn join(&self, other: &Self) -> Self {
        let mut new_map = rbt_map![];
        for (tag, val) in self.map.iter().chain(other.map.iter()) {
            let cur_val = *new_map.get(tag).unwrap_or(&BoolDomain::False);
            let new_val = cur_val.join(val);
            new_map.insert_mut(*tag, new_val);
        }
        TagDomain { map: new_map }
    }

    /// Return a tag domain element that filters out tags which are not propagated by an expression.
    #[logfn_inputs(TRACE)]
    pub fn filter(&self, exp_tag_prop: TagPropagation) -> Self {
        precondition!((exp_tag_prop as u8) < 128);
        let new_map: RedBlackTreeMap<Tag, BoolDomain> = self
            .map
            .iter()
            .filter_map(|(tag, val)| {
                let tag_propagation_set = tag.prop_set;
                if does_propagate_tag!(tag_propagation_set, exp_tag_prop) {
                    Some((*tag, *val))
                } else {
                    None
                }
            })
            .collect();
        TagDomain { map: new_map }
    }
}
