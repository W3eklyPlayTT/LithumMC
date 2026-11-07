use lithummc_macros::lithummc_block;

use crate::block::{BlockBehaviour, PathComputationType};

#[lithummc_block("minecraft:soul_sand")]
pub struct SoulSandBlock;

impl BlockBehaviour for SoulSandBlock {
    fn is_pathfindable(
        &self,
        _state: &lithummc_data::BlockState,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
}
