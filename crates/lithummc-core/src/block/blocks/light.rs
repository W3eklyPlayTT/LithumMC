use lithummc_data::BlockStateId;
use lithummc_data::block_properties::LightLikeProperties;
use lithummc_data::fluid::Fluid;
use lithummc_macros::lithummc_block;
use lithummc_util::{GameMode, PermissionLvl};
use lithummc_world::tick::TickPriority;
use lithummc_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, NormalUseArgs, OnPlaceArgs};

#[lithummc_block("minecraft:light")]
pub struct LightBlock;

impl BlockBehaviour for LightBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LightLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.gamemode.load() == GameMode::Creative
            && args.player.permission_lvl.load() >= PermissionLvl::Two
        {
            let state_id = args.world.get_block_state_id(args.position);
            let mut props = LightLikeProperties::from_state_id(state_id);
            props.level = (props.level + 1) % 16;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
            BlockActionResult::SuccessServer
        } else {
            BlockActionResult::Consume
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = LightLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }
}
