use crate::block::UseWithItemArgs;
use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use lithummc_data::Block;
use lithummc_data::block_properties::WallTorchLikeProperties;
use lithummc_data::entity::EntityType;
use lithummc_data::item::Item;
use lithummc_data::item_stack::ItemStack;
use lithummc_macros::lithummc_block;
use lithummc_world::world::BlockFlags;
use std::sync::Arc;

#[lithummc_block("minecraft:pumpkin")]
pub struct PumpkinBlock;

impl crate::block::BlockBehaviour for PumpkinBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.item != &Item::SHEARS {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        let mut props = WallTorchLikeProperties::default(&Block::CARVED_PUMPKIN);
        props.facing = args.hit.face.to_horizontal_facing().unwrap_or_else(|| {
            args.player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite()
        });
        args.world.play_sound(
            lithummc_data::sound::Sound::BlockPumpkinCarve,
            lithummc_data::sound::SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        args.world.set_block_state(
            args.position,
            props.to_state_id(&Block::CARVED_PUMPKIN),
            BlockFlags::NOTIFY_ALL,
        );
        let entity = Entity::new(
            args.world.clone(),
            args.position.to_centered_f64(),
            &EntityType::ITEM,
        );
        let item_entity = Arc::new(ItemEntity::new(
            entity,
            ItemStack::new(4, &Item::PUMPKIN_SEEDS),
        ));
        args.world.spawn_entity(item_entity);
        args.player.damage_item_in_slot(args.equipment_slot, 1);
        args.world.emit_game_event(
            lithummc_data::game_event::GameEvent::Shear.name(),
            args.position.to_centered_f64(),
        );
        args.player.increment_stat(
            lithummc_data::statistic::StatisticCategory::Used,
            i32::from(Item::SHEARS.id),
            1,
        );
        BlockActionResult::Success
    }
}
