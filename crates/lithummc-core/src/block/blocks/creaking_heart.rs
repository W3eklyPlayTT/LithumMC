use std::sync::Arc;

use lithummc_data::BlockStateId;
use lithummc_data::block_properties::{
    Axis, CreakingHeartLikeProperties, CreakingHeartState, PaleOakWoodLikeProperties,
};
use lithummc_data::sound::{Sound, SoundCategory};
use lithummc_data::tag::{self, Taggable};
use lithummc_macros::lithummc_block;
use lithummc_world::world::BlockFlags;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{BlockBehaviour, BrokenArgs, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};

#[lithummc_block("minecraft:creaking_heart")]
pub struct CreakingHeartBlock;

impl CreakingHeartBlock {
    fn is_aligned_pale_oak_log(state_id: BlockStateId, axis: Axis) -> bool {
        let block = state_id.to_block();
        block.has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
            && PaleOakWoodLikeProperties::from_state_id(state_id).axis == axis
    }

    fn check_active_logs(
        world: &dyn lithummc_world::world::BlockAccessor,
        pos: &lithummc_util::math::position::BlockPos,
        axis: Axis,
    ) -> bool {
        let (pos_a, pos_b) = match axis {
            Axis::X => (pos.west(), pos.east()),
            Axis::Y => (pos.down(), pos.up()),
            Axis::Z => (pos.north(), pos.south()),
        };

        let state_a = world.get_block_state_id(&pos_a);
        let state_b = world.get_block_state_id(&pos_b);

        Self::is_aligned_pale_oak_log(state_a, axis) && Self::is_aligned_pale_oak_log(state_b, axis)
    }
}

impl BlockBehaviour for CreakingHeartBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CreakingHeartLikeProperties::from_state_id(args.block.default_state.id);
        props.axis = match args.direction {
            lithummc_data::BlockDirection::North | lithummc_data::BlockDirection::South => Axis::Z,
            lithummc_data::BlockDirection::East | lithummc_data::BlockDirection::West => Axis::X,
            lithummc_data::BlockDirection::Up | lithummc_data::BlockDirection::Down => Axis::Y,
        };
        props.creaking_heart_state = CreakingHeartState::Uprooted;
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = CreakingHeartBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));

            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CreakingHeartLikeProperties::from_state_id(state_id);

            if Self::check_active_logs(args.world.as_ref(), args.position, props.axis) {
                props.creaking_heart_state = CreakingHeartState::Dormant;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );

                args.world.play_sound(
                    Sound::BlockCreakingHeartSpawn,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
            }
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = CreakingHeartLikeProperties::from_state_id(state_id);

            let active_logs =
                Self::check_active_logs(args.world.as_ref(), args.position, props.axis);
            let new_state = if active_logs {
                if props.creaking_heart_state == CreakingHeartState::Uprooted {
                    args.world.play_sound(
                        Sound::BlockCreakingHeartSpawn,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                    );
                }
                CreakingHeartState::Dormant
            } else {
                CreakingHeartState::Uprooted
            };

            if props.creaking_heart_state != new_state {
                props.creaking_heart_state = new_state;
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.play_sound(
            Sound::BlockCreakingHeartBreak,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
    }
}
