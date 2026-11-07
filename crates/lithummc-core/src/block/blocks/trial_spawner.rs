use std::sync::Arc;

use lithummc_macros::lithummc_block;

use crate::block::entities::trial_spawner::TrialSpawnerBlockEntity;
use crate::block::{BlockBehaviour, BrokenArgs, PlacedArgs};

#[lithummc_block("minecraft:trial_spawner")]
pub struct TrialSpawnerBlock;

impl BlockBehaviour for TrialSpawnerBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = TrialSpawnerBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.remove_block_entity(args.position);
    }
}
