use lithummc_macros::lithummc_block;

use crate::block::{BlockBehaviour, PathComputationType};

#[lithummc_block("minecraft:mud")]
pub struct MudBlock;

impl BlockBehaviour for MudBlock {
    fn is_pathfindable(
        &self,
        _state: &lithummc_data::BlockState,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
}
