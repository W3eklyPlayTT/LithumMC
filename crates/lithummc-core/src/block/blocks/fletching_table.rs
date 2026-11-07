use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs};

use lithummc_macros::lithummc_block;

#[lithummc_block("minecraft:fletching_table")]
pub struct FletchingTableBlock;

impl BlockBehaviour for FletchingTableBlock {
    fn normal_use(&self, _args: NormalUseArgs<'_>) -> BlockActionResult {
        BlockActionResult::Pass
    }
}
