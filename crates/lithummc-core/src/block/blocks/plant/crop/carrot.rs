use lithummc_data::BlockStateId;
use lithummc_macros::lithummc_block;
use lithummc_util::math::position::BlockPos;
use lithummc_world::world::BlockAccessor;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
    RandomTickArgs,
};

#[lithummc_block("minecraft:carrots")]
pub struct CarrotBlock;

impl BlockBehaviour for CarrotBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::is_valid_bonemeal_target(self, args.world, args.position)
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        <Self as CropBlockBase>::perform_bonemeal(self, args.world, args.position);
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_place_at(self, &args)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as CropBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        <Self as CropBlockBase>::on_entity_collision(self, &args);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        <Self as CropBlockBase>::random_tick(self, args.world, args.position);
    }
}

impl PlantBlockBase for CarrotBlock {
    // Crops require farmland below; without this override the generic plant
    // survival check (`supports_vegetation`) keeps them alive on dirt.
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as CropBlockBase>::can_plant_crop_on_top(self, block_accessor, pos)
    }
}

impl CropBlockBase for CarrotBlock {}
