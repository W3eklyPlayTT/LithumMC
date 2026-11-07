use lithummc_macros::lithummc_block;

use crate::block::BlockBehaviour;

#[lithummc_block("minecraft:tinted_glass")]
pub struct TintedGlassBlock;

impl BlockBehaviour for TintedGlassBlock {}
