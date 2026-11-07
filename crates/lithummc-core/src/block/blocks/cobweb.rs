use crate::block::{BlockBehaviour, OnEntityCollisionArgs};
use crate::entity::EntityBase;
use lithummc_data::effect::StatusEffect;
use lithummc_macros::lithummc_block;
use lithummc_util::math::vector3::Vector3;

#[lithummc_block(Block::COBWEB)]
pub struct CobwebBlock;

impl BlockBehaviour for CobwebBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let entity = args.entity.get_entity();
        let vec = if let Some(living) = entity.get_living_entity()
            && living.has_effect(&StatusEffect::WEAVING)
        {
            Vector3::new(0.5, 0.25, 0.5)
        } else {
            Vector3::new(0.25, 0.05, 0.25)
        };
        entity.slow_movement(args.state, vec);
    }
}
