use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::GetInsideCollisionShapeArgs;
use crate::block::OnEntityCollisionArgs;
use crate::block::PlacedArgs;
use crate::block::entities::end_portal::EndPortalBlockEntity;
use lithummc_data::dimension::Dimension;
use lithummc_macros::lithummc_block;
use lithummc_util::math::boundingbox::BoundingBox;
use lithummc_util::math::vector3::Vector3;

#[lithummc_block("minecraft:end_portal")]
pub struct EndPortalBlock;

impl BlockBehaviour for EndPortalBlock {
    fn get_inside_collision_shape(&self, _args: GetInsideCollisionShapeArgs<'_>) -> BoundingBox {
        BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.75, 1.0))
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let target_world =
            if args.world.dimension.minecraft_name == Dimension::THE_END.minecraft_name {
                args.server.get_world_from_dimension(&Dimension::OVERWORLD)
            } else {
                args.server.get_world_from_dimension(&Dimension::THE_END)
            };
        if Arc::ptr_eq(&target_world, args.world) {
            return;
        }
        args.entity
            .get_entity()
            .try_use_portal(target_world, *args.position);
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let nbt = EndPortalBlockEntity::create_nbt(*args.position);
        args.world.add_block_entity_nbt(*args.position, &nbt);

        args.world
            .add_block_entity(Arc::new(EndPortalBlockEntity::new(*args.position)));
    }
}
