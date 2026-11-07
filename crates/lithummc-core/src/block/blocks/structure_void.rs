use lithummc_macros::lithummc_block;

use crate::block::BlockBehaviour;

#[lithummc_block("minecraft:structure_void")]
pub struct StructureVoidBlock;

impl BlockBehaviour for StructureVoidBlock {}
