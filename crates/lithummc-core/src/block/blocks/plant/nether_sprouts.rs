use crate::block::{BlockBehaviour, CanPlaceAtArgs};
use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use lithummc_data::BlockStateId;
use lithummc_data::tag::{self, Taggable};
use lithummc_macros::lithummc_block;
#[lithummc_block("minecraft:nether_sprouts")]
pub struct NetherSproutsBlock;

impl BlockBehaviour for NetherSproutsBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }
    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }
}
impl PlantBlockBase for NetherSproutsBlock {
    fn can_plant_on_top(
        &self,
        block_accessor: &dyn lithummc_world::world::BlockAccessor,
        pos: &lithummc_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos);
        block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_NETHER_SPROUTS)
    }
}
