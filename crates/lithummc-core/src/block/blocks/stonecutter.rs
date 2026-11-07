use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetScreenHandlerFactoryArgs, NormalUseArgs, OnPlaceArgs, PathComputationType,
};

use lithummc_data::block_properties::WallTorchLikeProperties;
use lithummc_data::{BlockState, BlockStateId, translation};
use lithummc_inventory::player::player_inventory::PlayerInventory;
use lithummc_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use lithummc_macros::lithummc_block;
use lithummc_util::text::TextComponent;
use std::sync::Arc;
use std::sync::Mutex;

use lithummc_inventory::stonecutter_screen_handler::StonecutterScreenHandler;

#[lithummc_block("minecraft:stonecutter")]
pub struct StonecutterBlock;

impl BlockBehaviour for StonecutterBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = WallTorchLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                lithummc_data::statistic::StatisticCategory::Custom,
                lithummc_data::statistic::CustomStatistic::InteractWithStonecutter as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }

        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let recipe_manager = args.server.recipe_manager.clone();
        Some(Box::new(StonecutterScreenFactory(recipe_manager)))
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

struct StonecutterScreenFactory(Arc<crate::server::recipe::RecipeManager>);

impl ScreenHandlerFactory for StonecutterScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler: SharedScreenHandler = Arc::new(Mutex::new(StonecutterScreenHandler::new(
            sync_id,
            player_inventory,
            Some(self.0.clone()),
        )));
        Some(handler)
    }

    fn get_display_name(&self) -> TextComponent {
        lithummc_macros::translate_cross!(
            translation::java::CONTAINER_STONECUTTER,
            translation::bedrock::CONTAINER_STONECUTTER
        )
    }
}
