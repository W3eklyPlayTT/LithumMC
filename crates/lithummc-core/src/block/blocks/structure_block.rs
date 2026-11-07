use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, OnPlaceArgs};

use lithummc_data::BlockStateId;
use lithummc_data::block_properties::StructureBlockLikeProperties;
use lithummc_macros::lithummc_block;
use lithummc_util::PermissionLvl;

#[lithummc_block("minecraft:structure_block")]
pub struct StructureBlock;

impl BlockBehaviour for StructureBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let props = StructureBlockLikeProperties::default(args.block);
        props.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        {
            if args.player.permission_lvl.load() < PermissionLvl::Two {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            args.world.update_block_entity(&block_entity);

            BlockActionResult::Success
        }
    }
}
