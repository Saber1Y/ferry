//! An in-memory compact block cache for `zcash_client_backend::sync::run`.
//!
//! The sync loop downloads compact blocks into a cache, scans them, then deletes them, so the
//! cache only ever holds the batch being scanned. Keeping it in memory means FERRY writes no
//! block data to disk.

use std::collections::BTreeMap;
use std::sync::Mutex;

use async_trait::async_trait;
use zcash_client_backend::data_api::chain::{BlockCache, BlockSource, error};
use zcash_client_backend::data_api::scanning::ScanRange;
use zcash_client_backend::proto::compact_formats::CompactBlock;
use zcash_protocol::consensus::BlockHeight;

/// The cache's error type. Holding blocks in memory cannot fail, except through a poisoned lock.
#[derive(Debug, thiserror::Error)]
#[error("the block cache lock was poisoned")]
pub struct CacheError;

/// Compact blocks keyed by height.
#[derive(Default)]
pub struct MemoryBlockCache {
    blocks: Mutex<BTreeMap<BlockHeight, CompactBlock>>,
}

impl MemoryBlockCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, BTreeMap<BlockHeight, CompactBlock>>, CacheError> {
        self.blocks.lock().map_err(|_| CacheError)
    }
}

fn height_of(block: &CompactBlock) -> BlockHeight {
    BlockHeight::from_u32(u32::try_from(block.height).expect("block heights fit in u32"))
}

impl BlockSource for MemoryBlockCache {
    type Error = CacheError;

    fn with_blocks<F, WalletErrT>(
        &self,
        from_height: Option<BlockHeight>,
        limit: Option<usize>,
        mut with_block: F,
    ) -> Result<(), error::Error<WalletErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), error::Error<WalletErrT, Self::Error>>,
    {
        // Copy the batch out so the callback (which scans) never runs under the lock.
        let batch: Vec<CompactBlock> = {
            let blocks = self.lock().map_err(error::Error::BlockSource)?;
            let start = from_height.unwrap_or(BlockHeight::from_u32(0));
            blocks
                .range(start..)
                .take(limit.unwrap_or(usize::MAX))
                .map(|(_, block)| block.clone())
                .collect()
        };
        for block in batch {
            with_block(block)?;
        }
        Ok(())
    }
}

#[async_trait]
impl BlockCache for MemoryBlockCache {
    fn get_tip_height(&self, range: Option<&ScanRange>) -> Result<Option<BlockHeight>, Self::Error> {
        let blocks = self.lock()?;
        Ok(match range {
            Some(range) => blocks
                .range(range.block_range().clone())
                .next_back()
                .map(|(h, _)| *h),
            None => blocks.keys().next_back().copied(),
        })
    }

    async fn read(&self, range: &ScanRange) -> Result<Vec<CompactBlock>, Self::Error> {
        Ok(self
            .lock()?
            .range(range.block_range().clone())
            .map(|(_, block)| block.clone())
            .collect())
    }

    async fn insert(&self, compact_blocks: Vec<CompactBlock>) -> Result<(), Self::Error> {
        let mut blocks = self.lock()?;
        for block in compact_blocks {
            blocks.insert(height_of(&block), block);
        }
        Ok(())
    }

    async fn delete(&self, range: ScanRange) -> Result<(), Self::Error> {
        let mut blocks = self.lock()?;
        let doomed: Vec<BlockHeight> = blocks
            .range(range.block_range().clone())
            .map(|(h, _)| *h)
            .collect();
        for h in doomed {
            blocks.remove(&h);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_client_backend::data_api::scanning::ScanPriority;

    fn block(height: u32) -> CompactBlock {
        CompactBlock {
            height: u64::from(height),
            ..Default::default()
        }
    }

    fn range(start: u32, end: u32) -> ScanRange {
        ScanRange::from_parts(
            BlockHeight::from_u32(start)..BlockHeight::from_u32(end),
            ScanPriority::Historic,
        )
    }

    #[tokio::test]
    async fn insert_read_delete_in_height_order() {
        let cache = MemoryBlockCache::new();
        cache.insert(vec![block(5), block(3), block(4)]).await.unwrap();

        let read: Vec<u64> = cache.read(&range(3, 5)).await.unwrap().iter().map(|b| b.height).collect();
        assert_eq!(read, vec![3, 4]);
        assert_eq!(cache.get_tip_height(None).unwrap(), Some(BlockHeight::from_u32(5)));
        assert_eq!(
            cache.get_tip_height(Some(&range(3, 5))).unwrap(),
            Some(BlockHeight::from_u32(4))
        );

        let mut seen = vec![];
        cache
            .with_blocks::<_, ()>(Some(BlockHeight::from_u32(4)), Some(5), |b| {
                seen.push(b.height);
                Ok(())
            })
            .unwrap();
        assert_eq!(seen, vec![4, 5]);

        cache.delete(range(3, 5)).await.unwrap();
        assert_eq!(cache.get_tip_height(None).unwrap(), Some(BlockHeight::from_u32(5)));
        assert!(cache.read(&range(3, 5)).await.unwrap().is_empty());
    }
}
