use lithummc_protocol::codec::recipe::DynamicRecipe;

pub trait RecipeProvider: Send + Sync {
    fn get_dynamic_recipes(&self) -> Vec<DynamicRecipe>;
}

#[derive(Clone, Copy)]
pub enum GenericRecipe<'a> {
    Vanilla(&'a lithummc_data::recipes::CraftingRecipeTypes),
    Dynamic(&'a lithummc_protocol::codec::recipe::OwnedCraftingRecipe),
}

#[derive(Clone, Copy)]
pub enum IngredientRef<'a> {
    Vanilla(&'a lithummc_data::recipes::RecipeIngredientTypes),
    Dynamic(&'a lithummc_protocol::codec::recipe::OwnedRecipeIngredient),
}

impl IngredientRef<'_> {
    #[must_use]
    pub fn match_item(&self, item: &lithummc_data::item::Item) -> bool {
        match self {
            Self::Vanilla(v) => v.match_item(item),
            Self::Dynamic(d) => d.match_item(item),
        }
    }
}
