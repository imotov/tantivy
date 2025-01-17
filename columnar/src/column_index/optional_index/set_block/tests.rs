use std::collections::HashMap;

use crate::column_index::optional_index::set_block::set_block::DENSE_BLOCK_NUM_BYTES;
use crate::column_index::optional_index::set_block::{DenseBlockCodec, SparseBlockCodec};
use crate::column_index::optional_index::{Set, SetCodec};

fn test_set_helper<C: SetCodec<Item = u16>>(vals: &[u16]) -> usize {
    let mut buffer = Vec::new();
    C::serialize(vals.iter().copied(), &mut buffer).unwrap();
    let tested_set = C::open(buffer.as_slice());
    let hash_set: HashMap<C::Item, C::Item> = vals
        .iter()
        .copied()
        .enumerate()
        .map(|(ord, val)| (val, C::Item::try_from(ord).ok().unwrap()))
        .collect();
    for val in 0u16..=u16::MAX {
        assert_eq!(tested_set.contains(val), hash_set.contains_key(&val));
        assert_eq!(tested_set.rank_if_exists(val), hash_set.get(&val).copied());
    }
    for rank in 0..vals.len() {
        assert_eq!(tested_set.select(rank as u16), vals[rank]);
    }
    buffer.len()
}

#[test]
fn test_dense_block_set_u16_empty() {
    let buffer_len = test_set_helper::<DenseBlockCodec>(&[]);
    assert_eq!(buffer_len, DENSE_BLOCK_NUM_BYTES as usize);
}

#[test]
fn test_dense_block_set_u16_max() {
    let buffer_len = test_set_helper::<DenseBlockCodec>(&[u16::MAX]);
    assert_eq!(buffer_len, DENSE_BLOCK_NUM_BYTES as usize);
}

#[test]
fn test_sparse_block_set_u16_empty() {
    let buffer_len = test_set_helper::<SparseBlockCodec>(&[]);
    assert_eq!(buffer_len, 0);
}

#[test]
fn test_sparse_block_set_u16_max() {
    let buffer_len = test_set_helper::<SparseBlockCodec>(&[u16::MAX]);
    assert_eq!(buffer_len, 2);
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_prop_test_dense(els in proptest::collection::btree_set(0..=u16::MAX, 0..=u16::MAX as usize)) {
        let vals: Vec<u16> = els.into_iter().collect();
        let buffer_len = test_set_helper::<DenseBlockCodec>(&vals);
        assert_eq!(buffer_len, DENSE_BLOCK_NUM_BYTES as usize);
    }

    #[test]
    fn test_prop_test_sparse(els in proptest::collection::btree_set(0..=u16::MAX, 0..=u16::MAX as usize)) {
        let vals: Vec<u16> = els.into_iter().collect();
        let buffer_len = test_set_helper::<SparseBlockCodec>(&vals);
        assert_eq!(buffer_len, vals.len() * 2);
    }
}

#[test]
fn test_simple_translate_codec_codec_idx_to_original_idx_dense() {
    let mut buffer = Vec::new();
    DenseBlockCodec::serialize([1, 3, 17, 32, 30_000, 30_001].iter().copied(), &mut buffer)
        .unwrap();
    let tested_set = DenseBlockCodec::open(buffer.as_slice());
    assert!(tested_set.contains(1));
    assert_eq!(
        &tested_set
            .select_iter([0, 1, 2, 5].iter().copied())
            .collect::<Vec<u16>>(),
        &[1, 3, 17, 30_001]
    );
}

#[test]
fn test_simple_translate_codec_idx_to_original_idx_sparse() {
    let mut buffer = Vec::new();
    SparseBlockCodec::serialize([1, 3, 17].iter().copied(), &mut buffer).unwrap();
    let tested_set = SparseBlockCodec::open(buffer.as_slice());
    assert!(tested_set.contains(1));
    assert_eq!(
        &tested_set
            .select_iter([0, 1, 2].iter().copied())
            .collect::<Vec<u16>>(),
        &[1, 3, 17]
    );
}

#[test]
fn test_simple_translate_codec_idx_to_original_idx_dense() {
    let mut buffer = Vec::new();
    DenseBlockCodec::serialize(0u16..150u16, &mut buffer).unwrap();
    let tested_set = DenseBlockCodec::open(buffer.as_slice());
    assert!(tested_set.contains(1));
    let rg = 0u16..150u16;
    let els: Vec<u16> = rg.clone().collect();
    assert_eq!(
        &tested_set.select_iter(rg.clone()).collect::<Vec<u16>>(),
        &els
    );
}
