use std::sync::Arc;
use std::sync::Mutex;

use lithummc_data::translation;
use lithummc_inventory::Inventory;
use lithummc_inventory::beacon_screen_handler::create_beacon_handler;
use lithummc_inventory::player::player_inventory::PlayerInventory;
use lithummc_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use lithummc_macros::lithummc_block;
use lithummc_util::text::TextComponent;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, GetScreenHandlerFactoryArgs, NormalUseArgs};

// Create the factory just like ChestScreenFactory
struct BeaconScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for BeaconScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let concrete_handler = create_beacon_handler(sync_id, player_inventory, self.0.clone());
        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        lithummc_macros::translate_cross!(
            translation::java::CONTAINER_BEACON,
            translation::bedrock::CONTAINER_BEACON
        )
    }
}

#[lithummc_block("minecraft:beacon")]
pub struct BeaconBlock;

impl BlockBehaviour for BeaconBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        })
        .map_or(BlockActionResult::Fail, |factory| {
            args.player.increment_stat(
                lithummc_data::statistic::StatisticCategory::Custom,
                lithummc_data::statistic::CustomStatistic::InteractWithBeacon as i32,
                1,
            );

            // Open the screen using the factory
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));

            BlockActionResult::Success
        })
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.get_inventory()?;
        Some(Box::new(BeaconScreenFactory(inventory)))
    }
}
