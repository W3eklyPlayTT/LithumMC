use crate::lithummc::plugin::status_effect;
use lithummc_wasm_host_common::state::PluginHostState;

impl status_effect::Host for PluginHostState {}

#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn from_wasm_status_effect_type(
    t: status_effect::StatusEffectType,
) -> lithummc_data::status_effect::EffectType {
    match t {
        status_effect::StatusEffectType::Speed => lithummc_data::status_effect::EffectType::Speed,
        status_effect::StatusEffectType::Slowness => {
            lithummc_data::status_effect::EffectType::Slowness
        }
        status_effect::StatusEffectType::Haste => lithummc_data::status_effect::EffectType::Haste,
        status_effect::StatusEffectType::MiningFatigue => {
            lithummc_data::status_effect::EffectType::MiningFatigue
        }
        status_effect::StatusEffectType::Strength => {
            lithummc_data::status_effect::EffectType::Strength
        }
        status_effect::StatusEffectType::InstantHealth => {
            lithummc_data::status_effect::EffectType::InstantHealth
        }
        status_effect::StatusEffectType::InstantDamage => {
            lithummc_data::status_effect::EffectType::InstantDamage
        }
        status_effect::StatusEffectType::JumpBoost => {
            lithummc_data::status_effect::EffectType::JumpBoost
        }
        status_effect::StatusEffectType::Nausea => lithummc_data::status_effect::EffectType::Nausea,
        status_effect::StatusEffectType::Regeneration => {
            lithummc_data::status_effect::EffectType::Regeneration
        }
        status_effect::StatusEffectType::Resistance => {
            lithummc_data::status_effect::EffectType::Resistance
        }
        status_effect::StatusEffectType::FireResistance => {
            lithummc_data::status_effect::EffectType::FireResistance
        }
        status_effect::StatusEffectType::WaterBreathing => {
            lithummc_data::status_effect::EffectType::WaterBreathing
        }
        status_effect::StatusEffectType::Invisibility => {
            lithummc_data::status_effect::EffectType::Invisibility
        }
        status_effect::StatusEffectType::Blindness => {
            lithummc_data::status_effect::EffectType::Blindness
        }
        status_effect::StatusEffectType::NightVision => {
            lithummc_data::status_effect::EffectType::NightVision
        }
        status_effect::StatusEffectType::Hunger => lithummc_data::status_effect::EffectType::Hunger,
        status_effect::StatusEffectType::Weakness => {
            lithummc_data::status_effect::EffectType::Weakness
        }
        status_effect::StatusEffectType::Poison => lithummc_data::status_effect::EffectType::Poison,
        status_effect::StatusEffectType::Wither => lithummc_data::status_effect::EffectType::Wither,
        status_effect::StatusEffectType::HealthBoost => {
            lithummc_data::status_effect::EffectType::HealthBoost
        }
        status_effect::StatusEffectType::Absorption => {
            lithummc_data::status_effect::EffectType::Absorption
        }
        status_effect::StatusEffectType::Saturation => {
            lithummc_data::status_effect::EffectType::Saturation
        }
        status_effect::StatusEffectType::Glowing => {
            lithummc_data::status_effect::EffectType::Glowing
        }
        status_effect::StatusEffectType::Levitation => {
            lithummc_data::status_effect::EffectType::Levitation
        }
        status_effect::StatusEffectType::Luck => lithummc_data::status_effect::EffectType::Luck,
        status_effect::StatusEffectType::Unluck => lithummc_data::status_effect::EffectType::Unluck,
        status_effect::StatusEffectType::SlowFalling => {
            lithummc_data::status_effect::EffectType::SlowFalling
        }
        status_effect::StatusEffectType::ConduitPower => {
            lithummc_data::status_effect::EffectType::ConduitPower
        }
        status_effect::StatusEffectType::DolphinsGrace => {
            lithummc_data::status_effect::EffectType::DolphinsGrace
        }
        status_effect::StatusEffectType::BadOmen => {
            lithummc_data::status_effect::EffectType::BadOmen
        }
        status_effect::StatusEffectType::HeroOfTheVillage => {
            lithummc_data::status_effect::EffectType::HeroOfTheVillage
        }
        status_effect::StatusEffectType::Darkness => {
            lithummc_data::status_effect::EffectType::Darkness
        }
        status_effect::StatusEffectType::TrialOmen => {
            lithummc_data::status_effect::EffectType::TrialOmen
        }
        status_effect::StatusEffectType::RaidOmen => {
            lithummc_data::status_effect::EffectType::RaidOmen
        }
        status_effect::StatusEffectType::WindCharged => {
            lithummc_data::status_effect::EffectType::WindCharged
        }
        status_effect::StatusEffectType::Weaving => {
            lithummc_data::status_effect::EffectType::Weaving
        }
        status_effect::StatusEffectType::Oozing => lithummc_data::status_effect::EffectType::Oozing,
        status_effect::StatusEffectType::Infested => {
            lithummc_data::status_effect::EffectType::Infested
        }
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn to_wasm_status_effect_type(
    t: lithummc_data::status_effect::EffectType,
) -> status_effect::StatusEffectType {
    match t {
        lithummc_data::status_effect::EffectType::Speed => status_effect::StatusEffectType::Speed,
        lithummc_data::status_effect::EffectType::Slowness => {
            status_effect::StatusEffectType::Slowness
        }
        lithummc_data::status_effect::EffectType::Haste => status_effect::StatusEffectType::Haste,
        lithummc_data::status_effect::EffectType::MiningFatigue => {
            status_effect::StatusEffectType::MiningFatigue
        }
        lithummc_data::status_effect::EffectType::Strength => {
            status_effect::StatusEffectType::Strength
        }
        lithummc_data::status_effect::EffectType::InstantHealth => {
            status_effect::StatusEffectType::InstantHealth
        }
        lithummc_data::status_effect::EffectType::InstantDamage => {
            status_effect::StatusEffectType::InstantDamage
        }
        lithummc_data::status_effect::EffectType::JumpBoost => {
            status_effect::StatusEffectType::JumpBoost
        }
        lithummc_data::status_effect::EffectType::Nausea => status_effect::StatusEffectType::Nausea,
        lithummc_data::status_effect::EffectType::Regeneration => {
            status_effect::StatusEffectType::Regeneration
        }
        lithummc_data::status_effect::EffectType::Resistance => {
            status_effect::StatusEffectType::Resistance
        }
        lithummc_data::status_effect::EffectType::FireResistance => {
            status_effect::StatusEffectType::FireResistance
        }
        lithummc_data::status_effect::EffectType::WaterBreathing => {
            status_effect::StatusEffectType::WaterBreathing
        }
        lithummc_data::status_effect::EffectType::Invisibility => {
            status_effect::StatusEffectType::Invisibility
        }
        lithummc_data::status_effect::EffectType::Blindness => {
            status_effect::StatusEffectType::Blindness
        }
        lithummc_data::status_effect::EffectType::NightVision => {
            status_effect::StatusEffectType::NightVision
        }
        lithummc_data::status_effect::EffectType::Hunger => status_effect::StatusEffectType::Hunger,
        lithummc_data::status_effect::EffectType::Weakness => {
            status_effect::StatusEffectType::Weakness
        }
        lithummc_data::status_effect::EffectType::Poison => status_effect::StatusEffectType::Poison,
        lithummc_data::status_effect::EffectType::Wither => status_effect::StatusEffectType::Wither,
        lithummc_data::status_effect::EffectType::HealthBoost => {
            status_effect::StatusEffectType::HealthBoost
        }
        lithummc_data::status_effect::EffectType::Absorption => {
            status_effect::StatusEffectType::Absorption
        }
        lithummc_data::status_effect::EffectType::Saturation => {
            status_effect::StatusEffectType::Saturation
        }
        lithummc_data::status_effect::EffectType::Glowing => {
            status_effect::StatusEffectType::Glowing
        }
        lithummc_data::status_effect::EffectType::Levitation => {
            status_effect::StatusEffectType::Levitation
        }
        lithummc_data::status_effect::EffectType::Luck => status_effect::StatusEffectType::Luck,
        lithummc_data::status_effect::EffectType::Unluck => status_effect::StatusEffectType::Unluck,
        lithummc_data::status_effect::EffectType::SlowFalling => {
            status_effect::StatusEffectType::SlowFalling
        }
        lithummc_data::status_effect::EffectType::ConduitPower => {
            status_effect::StatusEffectType::ConduitPower
        }
        lithummc_data::status_effect::EffectType::DolphinsGrace => {
            status_effect::StatusEffectType::DolphinsGrace
        }
        lithummc_data::status_effect::EffectType::BadOmen => {
            status_effect::StatusEffectType::BadOmen
        }
        lithummc_data::status_effect::EffectType::HeroOfTheVillage => {
            status_effect::StatusEffectType::HeroOfTheVillage
        }
        lithummc_data::status_effect::EffectType::Darkness => {
            status_effect::StatusEffectType::Darkness
        }
        lithummc_data::status_effect::EffectType::TrialOmen => {
            status_effect::StatusEffectType::TrialOmen
        }
        lithummc_data::status_effect::EffectType::RaidOmen => {
            status_effect::StatusEffectType::RaidOmen
        }
        lithummc_data::status_effect::EffectType::WindCharged => {
            status_effect::StatusEffectType::WindCharged
        }
        lithummc_data::status_effect::EffectType::Weaving => {
            status_effect::StatusEffectType::Weaving
        }
        lithummc_data::status_effect::EffectType::Oozing => status_effect::StatusEffectType::Oozing,
        lithummc_data::status_effect::EffectType::Infested => {
            status_effect::StatusEffectType::Infested
        }
    }
}

#[must_use]
pub fn to_wasm_status_effect_instance(
    effect: &lithummc_data::potion::Effect,
) -> Option<status_effect::StatusEffectInstance> {
    let name = effect
        .effect_type
        .minecraft_name
        .strip_prefix("minecraft:")
        .unwrap_or(effect.effect_type.minecraft_name);
    let effect_type_enum = lithummc_data::status_effect::EffectType::from_name(name)?;
    let wasm_type = to_wasm_status_effect_type(effect_type_enum);
    Some(status_effect::StatusEffectInstance {
        effect_type: wasm_type,
        duration: u32::try_from(effect.duration).unwrap_or(0),
        amplifier: effect.amplifier,
        ambient: effect.ambient,
        show_particles: effect.show_particles,
        show_icon: effect.show_icon,
    })
}
