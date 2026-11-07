use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::{CommandErrorType, DISPATCHER_PARSE_EXCEPTION};
use crate::node::attached::AttachedNode;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use lithummc_data::entity::EntityType;
use lithummc_data::translation;
use lithummc_protocol::java::client::play::SuggestionProviders;
use lithummc_util::identifier::Identifier;
use lithummc_util::text::TextComponent;
use std::any::Any;
use std::iter::Iterator;

pub static ENTITY_TYPE_REGISTRY: &Identifier = &Identifier::vanilla_static("entity_type");
pub static ENCHANTMENT_REGISTRY: &Identifier = &Identifier::vanilla_static("enchantment");
pub static MOB_EFFECT_REGISTRY: &Identifier = &Identifier::vanilla_static("mob_effect");
pub static DAMAGE_TYPE_REGISTRY: &Identifier = &Identifier::vanilla_static("damage_type");

static DAMAGE_TYPES: [lithummc_data::damage::DamageType; 51] = [
    lithummc_data::damage::DamageType::ARROW,
    lithummc_data::damage::DamageType::BAD_RESPAWN_POINT,
    lithummc_data::damage::DamageType::CACTUS,
    lithummc_data::damage::DamageType::CAMPFIRE,
    lithummc_data::damage::DamageType::CRAMMING,
    lithummc_data::damage::DamageType::DRAGON_BREATH,
    lithummc_data::damage::DamageType::DROWN,
    lithummc_data::damage::DamageType::DRY_OUT,
    lithummc_data::damage::DamageType::ENDER_PEARL,
    lithummc_data::damage::DamageType::EXPLOSION,
    lithummc_data::damage::DamageType::FALL,
    lithummc_data::damage::DamageType::FALLING_ANVIL,
    lithummc_data::damage::DamageType::FALLING_BLOCK,
    lithummc_data::damage::DamageType::FALLING_STALACTITE,
    lithummc_data::damage::DamageType::FIREBALL,
    lithummc_data::damage::DamageType::FIREWORKS,
    lithummc_data::damage::DamageType::FLY_INTO_WALL,
    lithummc_data::damage::DamageType::FREEZE,
    lithummc_data::damage::DamageType::GENERIC,
    lithummc_data::damage::DamageType::GENERIC_KILL,
    lithummc_data::damage::DamageType::HOT_FLOOR,
    lithummc_data::damage::DamageType::IN_FIRE,
    lithummc_data::damage::DamageType::IN_WALL,
    lithummc_data::damage::DamageType::INDIRECT_MAGIC,
    lithummc_data::damage::DamageType::LAVA,
    lithummc_data::damage::DamageType::LIGHTNING_BOLT,
    lithummc_data::damage::DamageType::MACE_SMASH,
    lithummc_data::damage::DamageType::MAGIC,
    lithummc_data::damage::DamageType::MOB_ATTACK,
    lithummc_data::damage::DamageType::MOB_ATTACK_NO_AGGRO,
    lithummc_data::damage::DamageType::MOB_PROJECTILE,
    lithummc_data::damage::DamageType::ON_FIRE,
    lithummc_data::damage::DamageType::OUT_OF_WORLD,
    lithummc_data::damage::DamageType::OUTSIDE_BORDER,
    lithummc_data::damage::DamageType::PLAYER_ATTACK,
    lithummc_data::damage::DamageType::PLAYER_EXPLOSION,
    lithummc_data::damage::DamageType::SONIC_BOOM,
    lithummc_data::damage::DamageType::SPEAR,
    lithummc_data::damage::DamageType::SPIT,
    lithummc_data::damage::DamageType::STALAGMITE,
    lithummc_data::damage::DamageType::STARVE,
    lithummc_data::damage::DamageType::STING,
    lithummc_data::damage::DamageType::SULFUR_CUBE_HOT,
    lithummc_data::damage::DamageType::SWEET_BERRY_BUSH,
    lithummc_data::damage::DamageType::THORNS,
    lithummc_data::damage::DamageType::THROWN,
    lithummc_data::damage::DamageType::TRIDENT,
    lithummc_data::damage::DamageType::UNATTRIBUTED_FIREBALL,
    lithummc_data::damage::DamageType::WIND_CHARGE,
    lithummc_data::damage::DamageType::WITHER,
    lithummc_data::damage::DamageType::WITHER_SKULL,
];

static ERROR_UNKNOWN_RESOURCE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
);

static ERROR_INVALID_RESOURCE_TYPE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
);

static ERROR_NOT_SUMMONABLE_ENTITY: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ENTITY_NOT_SUMMONABLE,
    translation::java::ENTITY_NOT_SUMMONABLE,
);

pub static ENTITY_TYPE_ARGUMENT: ResourceArgument =
    ResourceArgument(ENTITY_TYPE_REGISTRY, &|id: Identifier| {
        EntityType::from_name(id.path()).map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub static ENCHANTMENT_ARGUMENT: ResourceArgument =
    ResourceArgument(ENCHANTMENT_REGISTRY, &|id: Identifier| {
        lithummc_data::Enchantment::from_name(id.path())
            .or_else(|| lithummc_data::Enchantment::from_name(&id.to_string()))
            .map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub static MOB_EFFECT_ARGUMENT: ResourceArgument =
    ResourceArgument(MOB_EFFECT_REGISTRY, &|id: Identifier| {
        lithummc_data::effect::StatusEffect::from_name(id.path())
            .or_else(|| lithummc_data::effect::StatusEffect::from_minecraft_name(&id.to_string()))
            .map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub type DynamicDamageTypeResolver =
    Box<dyn Fn(&str) -> Option<&'static lithummc_data::damage::DamageType> + Send + Sync>;

pub static DYNAMIC_DAMAGE_TYPE_RESOLVER: std::sync::RwLock<Option<DynamicDamageTypeResolver>> =
    std::sync::RwLock::new(None);
pub static DYNAMIC_DAMAGE_TYPE_NAMES: std::sync::RwLock<Vec<String>> =
    std::sync::RwLock::new(Vec::new());

pub fn register_dynamic_damage_types(resolver: DynamicDamageTypeResolver, names: Vec<String>) {
    if let Ok(mut guard) = DYNAMIC_DAMAGE_TYPE_RESOLVER.write() {
        *guard = Some(resolver);
    }
    if let Ok(mut guard) = DYNAMIC_DAMAGE_TYPE_NAMES.write() {
        *guard = names;
    }
}

pub static DAMAGE_TYPE_ARGUMENT: ResourceArgument =
    ResourceArgument(DAMAGE_TYPE_REGISTRY, &|id: Identifier| {
        if let Ok(guard) = DYNAMIC_DAMAGE_TYPE_RESOLVER.read()
            && let Some(resolver) = guard.as_ref()
        {
            let id_str = id.to_string();
            let path_str = id.path();
            if let Some(dt) = resolver(&id_str).or_else(|| resolver(path_str)) {
                return Some(dt as &'static (dyn Any + Send + Sync));
            }
        }

        let dt = lithummc_data::damage::DamageType::from_name(id.path())
            .or_else(|| lithummc_data::damage::DamageType::from_name(&id.to_string()))?;
        let idx = dt.id as usize;
        DAMAGE_TYPES
            .get(idx)
            .map(|val| val as &'static (dyn Any + Send + Sync))
    });

#[derive(Clone)]
pub struct ResourceArgument(
    pub &'static Identifier,
    pub &'static (dyn Fn(Identifier) -> Option<&'static (dyn Any + Send + Sync)> + Send + Sync),
);

impl<S: crate::source::CommandSource> ArgumentType<S> for ResourceArgument {
    type Item = &'static (dyn Any + Send + Sync);

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        self.1(identifier.clone()).ok_or_else(|| {
            ERROR_UNKNOWN_RESOURCE.create(
                reader,
                TextComponent::text(identifier.path().to_string()),
                TextComponent::text(self.0.to_string()),
            )
        })
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        suggestions_builder: SuggestionsBuilder,
    ) -> Suggestions {
        if self.0 == ENTITY_TYPE_REGISTRY {
            let entity_types = EntityType::ALL
                .iter()
                .filter(|entity_type| entity_type.summonable)
                .map(|entity_type| format!("minecraft:{}", entity_type.resource_name));
            suggestions_builder
                .filter_and_suggest_iter(entity_types)
                .build()
        } else if self.0 == ENCHANTMENT_REGISTRY {
            let enchantments = lithummc_data::Enchantment::ALL
                .iter()
                .map(|e| format!("minecraft:{}", e.name));
            suggestions_builder
                .filter_and_suggest_iter(enchantments)
                .build()
        } else if self.0 == MOB_EFFECT_REGISTRY {
            let effects = [
                &lithummc_data::effect::StatusEffect::ABSORPTION,
                &lithummc_data::effect::StatusEffect::BAD_OMEN,
                &lithummc_data::effect::StatusEffect::BLINDNESS,
                &lithummc_data::effect::StatusEffect::BREATH_OF_THE_NAUTILUS,
                &lithummc_data::effect::StatusEffect::CONDUIT_POWER,
                &lithummc_data::effect::StatusEffect::DARKNESS,
                &lithummc_data::effect::StatusEffect::DOLPHINS_GRACE,
                &lithummc_data::effect::StatusEffect::FIRE_RESISTANCE,
                &lithummc_data::effect::StatusEffect::GLOWING,
                &lithummc_data::effect::StatusEffect::HASTE,
                &lithummc_data::effect::StatusEffect::HEALTH_BOOST,
                &lithummc_data::effect::StatusEffect::HERO_OF_THE_VILLAGE,
                &lithummc_data::effect::StatusEffect::HUNGER,
                &lithummc_data::effect::StatusEffect::INFESTED,
                &lithummc_data::effect::StatusEffect::INSTANT_DAMAGE,
                &lithummc_data::effect::StatusEffect::INSTANT_HEALTH,
                &lithummc_data::effect::StatusEffect::INVISIBILITY,
                &lithummc_data::effect::StatusEffect::JUMP_BOOST,
                &lithummc_data::effect::StatusEffect::LEVITATION,
                &lithummc_data::effect::StatusEffect::LUCK,
                &lithummc_data::effect::StatusEffect::MINING_FATIGUE,
                &lithummc_data::effect::StatusEffect::NAUSEA,
                &lithummc_data::effect::StatusEffect::NIGHT_VISION,
                &lithummc_data::effect::StatusEffect::OOZING,
                &lithummc_data::effect::StatusEffect::POISON,
                &lithummc_data::effect::StatusEffect::RAID_OMEN,
                &lithummc_data::effect::StatusEffect::REGENERATION,
                &lithummc_data::effect::StatusEffect::RESISTANCE,
                &lithummc_data::effect::StatusEffect::SATURATION,
                &lithummc_data::effect::StatusEffect::SLOW_FALLING,
                &lithummc_data::effect::StatusEffect::SLOWNESS,
                &lithummc_data::effect::StatusEffect::SPEED,
                &lithummc_data::effect::StatusEffect::STRENGTH,
                &lithummc_data::effect::StatusEffect::TRIAL_OMEN,
                &lithummc_data::effect::StatusEffect::UNLUCK,
                &lithummc_data::effect::StatusEffect::WATER_BREATHING,
                &lithummc_data::effect::StatusEffect::WEAKNESS,
                &lithummc_data::effect::StatusEffect::WEAVING,
                &lithummc_data::effect::StatusEffect::WIND_CHARGED,
                &lithummc_data::effect::StatusEffect::WITHER,
            ]
            .iter()
            .map(|e| e.minecraft_name.to_string());
            suggestions_builder.filter_and_suggest_iter(effects).build()
        } else if self.0 == DAMAGE_TYPE_REGISTRY {
            let types = DAMAGE_TYPES
                .iter()
                .map(|dt| format!("minecraft:{}", dt.message_id));
            let extra = DYNAMIC_DAMAGE_TYPE_NAMES
                .read()
                .map(|g| g.clone())
                .unwrap_or_default();
            suggestions_builder
                .filter_and_suggest_iter(types.chain(extra))
                .build()
        } else {
            Suggestions::empty()
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Resource {
            identifier: self.0.clone(),
        }
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        (self.0 == ENTITY_TYPE_REGISTRY).then_some(SuggestionProviders::SummonableEntities)
    }
}

impl ResourceArgument {
    pub fn get_resource<T: 'static, S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
        registry_key: &Identifier,
    ) -> Result<&'static T, CommandSyntaxError> {
        let missing_argument = DISPATCHER_PARSE_EXCEPTION.create_without_context(
            TextComponent::text(format!("Could not find argument with name '{name}'")),
        );
        let node = context
            .nodes
            .iter()
            .rev()
            .find_map(|parsed| {
                if let AttachedNode::Argument(cur) = &context.tree[parsed.node]
                    && cur.meta.name == name
                {
                    Some(cur)
                } else {
                    None
                }
            })
            .or_else(|| {
                context.tree.iter().find_map(|node| {
                    if let AttachedNode::Argument(cur) = node
                        && cur.meta.name == name
                    {
                        Some(cur)
                    } else {
                        None
                    }
                })
            })
            .ok_or(missing_argument.clone())?;
        let invalid_argument =
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(format!(
                "argument with name '{name}' isn't a ResourceArgument"
            )));
        let result_argument = node
            .meta
            .argument_type
            .as_any()
            .downcast_ref::<Self>()
            .ok_or(invalid_argument)?;
        let registry_name = result_argument.0;
        let identifier = context
            .arguments
            .get(name)
            .ok_or(missing_argument)?
            .range
            .substring_slice(context.input.as_str())
            .to_string();
        let err = ERROR_INVALID_RESOURCE_TYPE.create_without_context(
            TextComponent::text(identifier),
            TextComponent::text(registry_name.to_string()),
            TextComponent::text(registry_key.to_string()),
        );
        if registry_name == registry_key {
            context
                .get_argument::<&'static (dyn Any + Send + Sync)>(name)?
                .downcast_ref::<T>()
                .ok_or(err)
        } else {
            Err(err)
        }
    }

    pub fn get_entity_type<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)
    }

    pub fn get_enchantment<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static lithummc_data::Enchantment, CommandSyntaxError> {
        Self::get_resource(context, name, ENCHANTMENT_REGISTRY)
    }

    pub fn get_mob_effect<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static lithummc_data::effect::StatusEffect, CommandSyntaxError> {
        Self::get_resource(context, name, MOB_EFFECT_REGISTRY)
    }

    pub fn get_damage_type<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static lithummc_data::damage::DamageType, CommandSyntaxError> {
        Self::get_resource(context, name, DAMAGE_TYPE_REGISTRY)
    }

    pub fn get_summonable_entity_type<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        let val: &'static EntityType = Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)?;
        if val.summonable {
            Ok(val)
        } else {
            Err(ERROR_NOT_SUMMONABLE_ENTITY
                .create_without_context(TextComponent::text(val.resource_name)))
        }
    }
}
