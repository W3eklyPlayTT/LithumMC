use crate::forms::FormsHostStateExt;
use lithummc_protocol::bedrock::client::PackIdVersion;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use wasmtime::component::{Access, HasSelf, Resource};

use crate::lithummc::plugin::forms::Form;
use crate::lithummc::plugin::java_dialogs::Dialog;
use crate::{
    events::{from_wasm_game_mode, from_wasm_position, to_wasm_game_mode, to_wasm_position},
    living_entity::from_wit_damage_type,
    lithummc::{
        self,
        plugin::{
            damage_types::DamageType as WitDamageType,
            player::{Player, PlayerSkin, SkinParts},
            statistics::{
                CustomStatistic as WitCustomStatistic, StatisticCategory as WitStatisticCategory,
            },
            uuid::Uuid,
            world::World,
        },
    },
    uuid::UuidExt,
};
use lithummc_core::plugin::api::gui::PluginScreenHandler;
use lithummc_core::{
    entity::{
        EntityBase,
        player::{
            TitleMode,
            advancement::{AdvancementAward, PlayerAdvancement},
        },
    },
    net::DisconnectReason,
};
use lithummc_inventory::player::player_inventory::PlayerInventory;
use lithummc_protocol::Property;
use lithummc_protocol::bedrock::client::modal_form_request::CModalFormRequest;
use lithummc_protocol::java::client::dialog::DialogNBT;
use lithummc_util::permission::PermissionLvl;
use lithummc_util::translation::Locale;
use lithummc_wasm_host_common::{plugin::WasmPlugin, state::PluginHostState};
use std::str::FromStr;

use lithummc_protocol::bedrock::client::set_actor_data::{
    CSetActorData, MetadataValue, PropertySyncData, SyncedActorDataList, entity_data_key,
};
use lithummc_protocol::codec::var_ulong::VarULong;
use lithummc_util::version::{BedrockMinecraftVersion, JavaMinecraftVersion};

const fn from_wit_client_game_event(
    event: lithummc::plugin::player::ClientGameEvent,
) -> lithummc_protocol::java::client::play::GameEvent {
    use lithummc::plugin::player::ClientGameEvent as W;
    use lithummc_protocol::java::client::play::GameEvent as G;
    match event {
        W::NoRespawnBlockAvailable => G::NoRespawnBlockAvailable,
        W::BeginRaining => G::BeginRaining,
        W::EndRaining => G::EndRaining,
        W::ChangeGameMode => G::ChangeGameMode,
        W::WinGame => G::WinGame,
        W::DemoEvent => G::DemoEvent,
        W::ArrowHitPlayer => G::ArrowHitPlayer,
        W::RainLevelChange => G::RainLevelChange,
        W::ThunderLevelChange => G::ThunderLevelChange,
        W::PlayPufferfishStringSound => G::PlayPufferfishStringSound,
        W::PlayElderGuardianMobAppearance => G::PlayElderGuardianMobAppearance,
        W::EnabledRespawnScreen => G::EnabledRespawnScreen,
        W::LimitedCrafting => G::LimitedCrafting,
        W::StartWaitingChunks => G::StartWaitingChunks,
    }
}

pub(crate) fn from_wit_server_link(
    state: &mut PluginHostState,
    link: lithummc::plugin::player::ServerLink,
) -> wasmtime::Result<(lithummc_protocol::Label, String)> {
    use lithummc::plugin::player::{KnownServerLink, ServerLinkLabel};
    let label = match link.label {
        ServerLinkLabel::Known(known) => {
            let link_type = match known {
                KnownServerLink::BugReport => lithummc_protocol::LinkType::BugReport,
                KnownServerLink::CommunityGuidelines => {
                    lithummc_protocol::LinkType::CommunityGuidelines
                }
                KnownServerLink::Support => lithummc_protocol::LinkType::Support,
                KnownServerLink::Status => lithummc_protocol::LinkType::Status,
                KnownServerLink::Feedback => lithummc_protocol::LinkType::Feedback,
                KnownServerLink::Community => lithummc_protocol::LinkType::Community,
                KnownServerLink::Website => lithummc_protocol::LinkType::Website,
                KnownServerLink::Forums => lithummc_protocol::LinkType::Forums,
                KnownServerLink::News => lithummc_protocol::LinkType::News,
                KnownServerLink::Announcements => lithummc_protocol::LinkType::Announcements,
            };
            lithummc_protocol::Label::BuiltIn(link_type)
        }
        ServerLinkLabel::Custom(text) => {
            let text_res = state.take(text)?;
            lithummc_protocol::Label::TextComponent(Box::new(text_res))
        }
    };
    Ok((label, link.url))
}

const fn to_wasm_java_version(
    version: JavaMinecraftVersion,
) -> lithummc::plugin::player::JavaMinecraftVersion {
    match version {
        JavaMinecraftVersion::V_1_7_2 => lithummc::plugin::player::JavaMinecraftVersion::V172,
        JavaMinecraftVersion::V_1_7_6 => lithummc::plugin::player::JavaMinecraftVersion::V176,
        JavaMinecraftVersion::V_1_8 => lithummc::plugin::player::JavaMinecraftVersion::V18,
        JavaMinecraftVersion::V_1_9 => lithummc::plugin::player::JavaMinecraftVersion::V19,
        JavaMinecraftVersion::V_1_9_1 => lithummc::plugin::player::JavaMinecraftVersion::V191,
        JavaMinecraftVersion::V_1_9_2 => lithummc::plugin::player::JavaMinecraftVersion::V192,
        JavaMinecraftVersion::V_1_9_3 => lithummc::plugin::player::JavaMinecraftVersion::V193,
        JavaMinecraftVersion::V_1_10 => lithummc::plugin::player::JavaMinecraftVersion::V110,
        JavaMinecraftVersion::V_1_11 => lithummc::plugin::player::JavaMinecraftVersion::V111,
        JavaMinecraftVersion::V_1_11_1 => lithummc::plugin::player::JavaMinecraftVersion::V1111,
        JavaMinecraftVersion::V_1_12 => lithummc::plugin::player::JavaMinecraftVersion::V112,
        JavaMinecraftVersion::V_1_12_1 => lithummc::plugin::player::JavaMinecraftVersion::V1121,
        JavaMinecraftVersion::V_1_12_2 => lithummc::plugin::player::JavaMinecraftVersion::V1122,
        JavaMinecraftVersion::V_1_13 => lithummc::plugin::player::JavaMinecraftVersion::V113,
        JavaMinecraftVersion::V_1_13_1 => lithummc::plugin::player::JavaMinecraftVersion::V1131,
        JavaMinecraftVersion::V_1_13_2 => lithummc::plugin::player::JavaMinecraftVersion::V1132,
        JavaMinecraftVersion::V_1_14 => lithummc::plugin::player::JavaMinecraftVersion::V114,
        JavaMinecraftVersion::V_1_14_1 => lithummc::plugin::player::JavaMinecraftVersion::V1141,
        JavaMinecraftVersion::V_1_14_2 => lithummc::plugin::player::JavaMinecraftVersion::V1142,
        JavaMinecraftVersion::V_1_14_3 => lithummc::plugin::player::JavaMinecraftVersion::V1143,
        JavaMinecraftVersion::V_1_14_4 => lithummc::plugin::player::JavaMinecraftVersion::V1144,
        JavaMinecraftVersion::V_1_15 => lithummc::plugin::player::JavaMinecraftVersion::V115,
        JavaMinecraftVersion::V_1_15_1 => lithummc::plugin::player::JavaMinecraftVersion::V1151,
        JavaMinecraftVersion::V_1_15_2 => lithummc::plugin::player::JavaMinecraftVersion::V1152,
        JavaMinecraftVersion::V_1_16 => lithummc::plugin::player::JavaMinecraftVersion::V116,
        JavaMinecraftVersion::V_1_16_1 => lithummc::plugin::player::JavaMinecraftVersion::V1161,
        JavaMinecraftVersion::V_1_16_2 => lithummc::plugin::player::JavaMinecraftVersion::V1162,
        JavaMinecraftVersion::V_1_16_3 => lithummc::plugin::player::JavaMinecraftVersion::V1163,
        JavaMinecraftVersion::V_1_16_4 => lithummc::plugin::player::JavaMinecraftVersion::V1164,
        JavaMinecraftVersion::V_1_17 => lithummc::plugin::player::JavaMinecraftVersion::V117,
        JavaMinecraftVersion::V_1_17_1 => lithummc::plugin::player::JavaMinecraftVersion::V1171,
        JavaMinecraftVersion::V_1_18 => lithummc::plugin::player::JavaMinecraftVersion::V118,
        JavaMinecraftVersion::V_1_18_2 => lithummc::plugin::player::JavaMinecraftVersion::V1182,
        JavaMinecraftVersion::V_1_19 => lithummc::plugin::player::JavaMinecraftVersion::V119,
        JavaMinecraftVersion::V_1_19_1 => lithummc::plugin::player::JavaMinecraftVersion::V1191,
        JavaMinecraftVersion::V_1_19_3 => lithummc::plugin::player::JavaMinecraftVersion::V1193,
        JavaMinecraftVersion::V_1_19_4 => lithummc::plugin::player::JavaMinecraftVersion::V1194,
        JavaMinecraftVersion::V_1_20 => lithummc::plugin::player::JavaMinecraftVersion::V120,
        JavaMinecraftVersion::V_1_20_2 => lithummc::plugin::player::JavaMinecraftVersion::V1202,
        JavaMinecraftVersion::V_1_20_3 => lithummc::plugin::player::JavaMinecraftVersion::V1203,
        JavaMinecraftVersion::V_1_20_5 => lithummc::plugin::player::JavaMinecraftVersion::V1205,
        JavaMinecraftVersion::V_1_21 => lithummc::plugin::player::JavaMinecraftVersion::V121,
        JavaMinecraftVersion::V_1_21_2 => lithummc::plugin::player::JavaMinecraftVersion::V1212,
        JavaMinecraftVersion::V_1_21_4 => lithummc::plugin::player::JavaMinecraftVersion::V1214,
        JavaMinecraftVersion::V_1_21_5 => lithummc::plugin::player::JavaMinecraftVersion::V1215,
        JavaMinecraftVersion::V_1_21_6 => lithummc::plugin::player::JavaMinecraftVersion::V1216,
        JavaMinecraftVersion::V_1_21_7 => lithummc::plugin::player::JavaMinecraftVersion::V1217,
        JavaMinecraftVersion::V_1_21_9 => lithummc::plugin::player::JavaMinecraftVersion::V1219,
        JavaMinecraftVersion::V_1_21_11 => lithummc::plugin::player::JavaMinecraftVersion::V12111,
        JavaMinecraftVersion::V_26_1 => lithummc::plugin::player::JavaMinecraftVersion::V261,
        JavaMinecraftVersion::V_26_2 => lithummc::plugin::player::JavaMinecraftVersion::V262,
        JavaMinecraftVersion::V_26_3 => lithummc::plugin::player::JavaMinecraftVersion::V263,
        JavaMinecraftVersion::Unknown => lithummc::plugin::player::JavaMinecraftVersion::Unknown,
    }
}

const fn to_wasm_bedrock_version(
    version: BedrockMinecraftVersion,
) -> lithummc::plugin::player::BedrockMinecraftVersion {
    match version {
        BedrockMinecraftVersion::V_1_21 => lithummc::plugin::player::BedrockMinecraftVersion::V121,
        BedrockMinecraftVersion::V_1_26_51 | BedrockMinecraftVersion::V_1_26_60 => {
            // The v0.1 ABI does not define this Bedrock version.
            lithummc::plugin::player::BedrockMinecraftVersion::Unknown
        }
        BedrockMinecraftVersion::Unknown => {
            lithummc::plugin::player::BedrockMinecraftVersion::Unknown
        }
    }
}

const fn to_wasm_chat_mode(
    mode: &lithummc_core::entity::player::ChatMode,
) -> lithummc::plugin::player::ChatMode {
    match mode {
        lithummc_core::entity::player::ChatMode::Enabled => {
            lithummc::plugin::player::ChatMode::Enabled
        }
        lithummc_core::entity::player::ChatMode::CommandsOnly => {
            lithummc::plugin::player::ChatMode::CommandsOnly
        }
        lithummc_core::entity::player::ChatMode::Hidden => lithummc::plugin::player::ChatMode::Hidden,
    }
}

#[must_use]
pub const fn to_wit_statistic_category(
    category: lithummc_data::statistic::StatisticCategory,
) -> WitStatisticCategory {
    match category {
        lithummc_data::statistic::StatisticCategory::Mined => WitStatisticCategory::Mined,
        lithummc_data::statistic::StatisticCategory::Crafted => WitStatisticCategory::Crafted,
        lithummc_data::statistic::StatisticCategory::Used => WitStatisticCategory::Used,
        lithummc_data::statistic::StatisticCategory::Broken => WitStatisticCategory::Broken,
        lithummc_data::statistic::StatisticCategory::PickedUp => WitStatisticCategory::PickedUp,
        lithummc_data::statistic::StatisticCategory::Dropped => WitStatisticCategory::Dropped,
        lithummc_data::statistic::StatisticCategory::Killed => WitStatisticCategory::Killed,
        lithummc_data::statistic::StatisticCategory::KilledBy => WitStatisticCategory::KilledBy,
        lithummc_data::statistic::StatisticCategory::Custom => WitStatisticCategory::Custom,
    }
}

#[must_use]
pub const fn from_wit_statistic_category(
    wit: WitStatisticCategory,
) -> lithummc_data::statistic::StatisticCategory {
    match wit {
        WitStatisticCategory::Mined => lithummc_data::statistic::StatisticCategory::Mined,
        WitStatisticCategory::Crafted => lithummc_data::statistic::StatisticCategory::Crafted,
        WitStatisticCategory::Used => lithummc_data::statistic::StatisticCategory::Used,
        WitStatisticCategory::Broken => lithummc_data::statistic::StatisticCategory::Broken,
        WitStatisticCategory::PickedUp => lithummc_data::statistic::StatisticCategory::PickedUp,
        WitStatisticCategory::Dropped => lithummc_data::statistic::StatisticCategory::Dropped,
        WitStatisticCategory::Killed => lithummc_data::statistic::StatisticCategory::Killed,
        WitStatisticCategory::KilledBy => lithummc_data::statistic::StatisticCategory::KilledBy,
        WitStatisticCategory::Custom => lithummc_data::statistic::StatisticCategory::Custom,
    }
}

#[must_use]
pub const fn to_wit_custom_statistic(
    stat: lithummc_data::statistic::CustomStatistic,
) -> WitCustomStatistic {
    // SAFETY: WitCustomStatistic is generated in the same numerical order as CustomStatistic
    unsafe { std::mem::transmute(stat as u8) }
}

#[must_use]
pub fn from_wit_custom_statistic(
    wit: WitCustomStatistic,
) -> lithummc_data::statistic::CustomStatistic {
    lithummc_data::statistic::CustomStatistic::from_i32(wit as i32)
        .unwrap_or(lithummc_data::statistic::CustomStatistic::PlayTime)
}

const fn to_wasm_bedrock_device_os(os: i32) -> lithummc::plugin::player::BedrockDeviceOs {
    match os {
        1 => lithummc::plugin::player::BedrockDeviceOs::Android,
        2 => lithummc::plugin::player::BedrockDeviceOs::Ios,
        3 => lithummc::plugin::player::BedrockDeviceOs::Osx,
        4 => lithummc::plugin::player::BedrockDeviceOs::Amazon,
        5 => lithummc::plugin::player::BedrockDeviceOs::GearVr,
        6 => lithummc::plugin::player::BedrockDeviceOs::HoloLens,
        7 => lithummc::plugin::player::BedrockDeviceOs::Windows10,
        8 => lithummc::plugin::player::BedrockDeviceOs::Win32,
        9 => lithummc::plugin::player::BedrockDeviceOs::Dedicated,
        10 => lithummc::plugin::player::BedrockDeviceOs::TvOs,
        11 => lithummc::plugin::player::BedrockDeviceOs::Playstation,
        12 => lithummc::plugin::player::BedrockDeviceOs::Nintendo,
        13 => lithummc::plugin::player::BedrockDeviceOs::Xbox,
        14 => lithummc::plugin::player::BedrockDeviceOs::WindowsPhone,
        15 => lithummc::plugin::player::BedrockDeviceOs::Linux,
        _ => lithummc::plugin::player::BedrockDeviceOs::Unknown,
    }
}

const fn to_wasm_bedrock_input_mode(mode: i32) -> lithummc::plugin::player::BedrockInputMode {
    match mode {
        1 => lithummc::plugin::player::BedrockInputMode::Mouse,
        2 => lithummc::plugin::player::BedrockInputMode::Touch,
        3 => lithummc::plugin::player::BedrockInputMode::GamePad,
        4 => lithummc::plugin::player::BedrockInputMode::MotionController,
        _ => lithummc::plugin::player::BedrockInputMode::Unknown,
    }
}

const fn to_wasm_bedrock_ui_profile(profile: i32) -> lithummc::plugin::player::BedrockUiProfile {
    match profile {
        0 => lithummc::plugin::player::BedrockUiProfile::Classic,
        1 => lithummc::plugin::player::BedrockUiProfile::Pocket,
        _ => lithummc::plugin::player::BedrockUiProfile::Unknown,
    }
}

const fn to_wasm_bedrock_graphics_mode(mode: i32) -> lithummc::plugin::player::BedrockGraphicsMode {
    match mode {
        0 => lithummc::plugin::player::BedrockGraphicsMode::Simple,
        1 => lithummc::plugin::player::BedrockGraphicsMode::Fancy,
        2 => lithummc::plugin::player::BedrockGraphicsMode::RayTraced,
        _ => lithummc::plugin::player::BedrockGraphicsMode::Unknown,
    }
}

const fn from_wasm_bedrock_ability(
    ability: lithummc::plugin::player::BedrockAbility,
) -> lithummc_protocol::bedrock::client::update_abilities::Ability {
    match ability {
        lithummc::plugin::player::BedrockAbility::Build => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Build
        }
        lithummc::plugin::player::BedrockAbility::Mine => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Mine
        }
        lithummc::plugin::player::BedrockAbility::DoorsAndSwitches => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::DoorsAndSwitches
        }
        lithummc::plugin::player::BedrockAbility::OpenContainers => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::OpenContainers
        }
        lithummc::plugin::player::BedrockAbility::AttackPlayers => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::AttackPlayers
        }
        lithummc::plugin::player::BedrockAbility::AttackMobs => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::AttackMobs
        }
        lithummc::plugin::player::BedrockAbility::OperatorCommands => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::OperatorCommands
        }
        lithummc::plugin::player::BedrockAbility::Teleport => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Teleport
        }
        lithummc::plugin::player::BedrockAbility::Invulnerable => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Invulnerable
        }
        lithummc::plugin::player::BedrockAbility::Flying => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Flying
        }
        lithummc::plugin::player::BedrockAbility::MayFly => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::MayFly
        }
        lithummc::plugin::player::BedrockAbility::Instabuild => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Instabuild
        }
        lithummc::plugin::player::BedrockAbility::Lightning => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Lightning
        }
        lithummc::plugin::player::BedrockAbility::FlySpeed => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::FlySpeed
        }
        lithummc::plugin::player::BedrockAbility::WalkSpeed => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::WalkSpeed
        }
        lithummc::plugin::player::BedrockAbility::Muted => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Muted
        }
        lithummc::plugin::player::BedrockAbility::WorldBuilder => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::WorldBuilder
        }
        lithummc::plugin::player::BedrockAbility::NoClip => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::NoClip
        }
        lithummc::plugin::player::BedrockAbility::PrivilegedBuilder => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::PrivilegedBuilder
        }
        lithummc::plugin::player::BedrockAbility::VerticalFlySpeed => {
            lithummc_protocol::bedrock::client::update_abilities::Ability::VerticalFlySpeed
        }
    }
}

#[allow(clippy::too_many_lines)]
const fn from_wasm_bedrock_status_flag(flag: lithummc::plugin::player::BedrockStatusFlag) -> u32 {
    use lithummc_protocol::bedrock::client::set_actor_data::entity_data_flag;
    match flag {
        lithummc::plugin::player::BedrockStatusFlag::OnFire => entity_data_flag::ON_FIRE,
        lithummc::plugin::player::BedrockStatusFlag::Sneaking => entity_data_flag::SNEAKING,
        lithummc::plugin::player::BedrockStatusFlag::Riding => entity_data_flag::RIDING,
        lithummc::plugin::player::BedrockStatusFlag::Sprinting => entity_data_flag::SPRINTING,
        lithummc::plugin::player::BedrockStatusFlag::UsingItem => entity_data_flag::USING_ITEM,
        lithummc::plugin::player::BedrockStatusFlag::Invisible => entity_data_flag::INVISIBLE,
        lithummc::plugin::player::BedrockStatusFlag::Tempted => entity_data_flag::TEMPTED,
        lithummc::plugin::player::BedrockStatusFlag::InLove => entity_data_flag::IN_LOVE,
        lithummc::plugin::player::BedrockStatusFlag::Saddled => entity_data_flag::SADDLED,
        lithummc::plugin::player::BedrockStatusFlag::Powered => entity_data_flag::POWERED,
        lithummc::plugin::player::BedrockStatusFlag::Ignited => entity_data_flag::IGNITED,
        lithummc::plugin::player::BedrockStatusFlag::Baby => entity_data_flag::BABY,
        lithummc::plugin::player::BedrockStatusFlag::Converting => entity_data_flag::CONVERTING,
        lithummc::plugin::player::BedrockStatusFlag::Critical => entity_data_flag::CRITICAL,
        lithummc::plugin::player::BedrockStatusFlag::ShowName => entity_data_flag::SHOW_NAME,
        lithummc::plugin::player::BedrockStatusFlag::AlwaysShowName => {
            entity_data_flag::ALWAYS_SHOW_NAME
        }
        lithummc::plugin::player::BedrockStatusFlag::NoAi => entity_data_flag::NO_AI,
        lithummc::plugin::player::BedrockStatusFlag::Silent => entity_data_flag::SILENT,
        lithummc::plugin::player::BedrockStatusFlag::WallClimbing => entity_data_flag::WALL_CLIMBING,
        lithummc::plugin::player::BedrockStatusFlag::Climb => entity_data_flag::CLIMB,
        lithummc::plugin::player::BedrockStatusFlag::Swim => entity_data_flag::SWIM,
        lithummc::plugin::player::BedrockStatusFlag::Fly => entity_data_flag::FLY,
        lithummc::plugin::player::BedrockStatusFlag::Walk => entity_data_flag::WALK,
        lithummc::plugin::player::BedrockStatusFlag::Resting => entity_data_flag::RESTING,
        lithummc::plugin::player::BedrockStatusFlag::Sitting => entity_data_flag::SITTING,
        lithummc::plugin::player::BedrockStatusFlag::Angry => entity_data_flag::ANGRY,
        lithummc::plugin::player::BedrockStatusFlag::Interested => entity_data_flag::INTERESTED,
        lithummc::plugin::player::BedrockStatusFlag::Charged => entity_data_flag::CHARGED,
        lithummc::plugin::player::BedrockStatusFlag::Tamed => entity_data_flag::TAMED,
        lithummc::plugin::player::BedrockStatusFlag::Orphaned => entity_data_flag::ORPHANED,
        lithummc::plugin::player::BedrockStatusFlag::Leashed => entity_data_flag::LEASHED,
        lithummc::plugin::player::BedrockStatusFlag::Sheared => entity_data_flag::SHEARED,
        lithummc::plugin::player::BedrockStatusFlag::Gliding => entity_data_flag::GLIDING,
        lithummc::plugin::player::BedrockStatusFlag::Elder => entity_data_flag::ELDER,
        lithummc::plugin::player::BedrockStatusFlag::Moving => entity_data_flag::MOVING,
        lithummc::plugin::player::BedrockStatusFlag::Breathing => entity_data_flag::BREATHING,
        lithummc::plugin::player::BedrockStatusFlag::Chested => entity_data_flag::CHESTED,
        lithummc::plugin::player::BedrockStatusFlag::Stackable => entity_data_flag::STACKABLE,
        lithummc::plugin::player::BedrockStatusFlag::ShowBottom => entity_data_flag::SHOW_BOTTOM,
        lithummc::plugin::player::BedrockStatusFlag::Standing => entity_data_flag::STANDING,
        lithummc::plugin::player::BedrockStatusFlag::Shaking => entity_data_flag::SHAKING,
        lithummc::plugin::player::BedrockStatusFlag::Idling => entity_data_flag::IDLING,
        lithummc::plugin::player::BedrockStatusFlag::Casting => entity_data_flag::CASTING,
        lithummc::plugin::player::BedrockStatusFlag::Charging => entity_data_flag::CHARGING,
        lithummc::plugin::player::BedrockStatusFlag::KeyboardControlled => {
            entity_data_flag::KEYBOARD_CONTROLLED
        }
        lithummc::plugin::player::BedrockStatusFlag::PowerJump => entity_data_flag::POWER_JUMP,
        lithummc::plugin::player::BedrockStatusFlag::Dash => entity_data_flag::DASH,
        lithummc::plugin::player::BedrockStatusFlag::Lingering => entity_data_flag::LINGERING,
        lithummc::plugin::player::BedrockStatusFlag::HasCollision => entity_data_flag::HAS_COLLISION,
        lithummc::plugin::player::BedrockStatusFlag::HasGravity => entity_data_flag::HAS_GRAVITY,
        lithummc::plugin::player::BedrockStatusFlag::FireImmune => entity_data_flag::FIRE_IMMUNE,
        lithummc::plugin::player::BedrockStatusFlag::Dancing => entity_data_flag::DANCING,
        lithummc::plugin::player::BedrockStatusFlag::Enchanted => entity_data_flag::ENCHANTED,
        lithummc::plugin::player::BedrockStatusFlag::ReturnTrident => {
            entity_data_flag::RETURN_TRIDENT
        }
        lithummc::plugin::player::BedrockStatusFlag::ContainerPrivate => {
            entity_data_flag::CONTAINER_PRIVATE
        }
        lithummc::plugin::player::BedrockStatusFlag::Transforming => entity_data_flag::TRANSFORMING,
        lithummc::plugin::player::BedrockStatusFlag::DamageNearbyMobs => {
            entity_data_flag::DAMAGE_NEARBY_MOBS
        }
        lithummc::plugin::player::BedrockStatusFlag::Swimming => entity_data_flag::SWIMMING,
        lithummc::plugin::player::BedrockStatusFlag::Bribed => entity_data_flag::BRIBED,
        lithummc::plugin::player::BedrockStatusFlag::Pregnant => entity_data_flag::PREGNANT,
        lithummc::plugin::player::BedrockStatusFlag::LayingEgg => entity_data_flag::LAYING_EGG,
        lithummc::plugin::player::BedrockStatusFlag::PassengerCanPick => {
            entity_data_flag::PASSENGER_CAN_PICK
        }
        lithummc::plugin::player::BedrockStatusFlag::TransitionSitting => {
            entity_data_flag::TRANSITION_SITTING
        }
        lithummc::plugin::player::BedrockStatusFlag::Eating => entity_data_flag::EATING,
        lithummc::plugin::player::BedrockStatusFlag::LayingDown => entity_data_flag::LAYING_DOWN,
        lithummc::plugin::player::BedrockStatusFlag::Sneezing => entity_data_flag::SNEEZING,
        lithummc::plugin::player::BedrockStatusFlag::Trusting => entity_data_flag::TRUSTING,
        lithummc::plugin::player::BedrockStatusFlag::Rolling => entity_data_flag::ROLLING,
        lithummc::plugin::player::BedrockStatusFlag::Scared => entity_data_flag::SCARED,
        lithummc::plugin::player::BedrockStatusFlag::InScaffolding => {
            entity_data_flag::IN_SCAFFOLDING
        }
        lithummc::plugin::player::BedrockStatusFlag::OverScaffolding => {
            entity_data_flag::OVER_SCAFFOLDING
        }
        lithummc::plugin::player::BedrockStatusFlag::DescendThroughBlock => {
            entity_data_flag::DESCEND_THROUGH_BLOCK
        }
        lithummc::plugin::player::BedrockStatusFlag::Blocking => entity_data_flag::BLOCKING,
        lithummc::plugin::player::BedrockStatusFlag::TransitionBlocking => {
            entity_data_flag::TRANSITION_BLOCKING
        }
        lithummc::plugin::player::BedrockStatusFlag::BlockedUsingShield => {
            entity_data_flag::BLOCKED_USING_SHIELD
        }
        lithummc::plugin::player::BedrockStatusFlag::BlockedUsingDamagedShield => {
            entity_data_flag::BLOCKED_USING_DAMAGED_SHIELD
        }
        lithummc::plugin::player::BedrockStatusFlag::Sleeping => entity_data_flag::SLEEPING,
        lithummc::plugin::player::BedrockStatusFlag::WantsToWake => entity_data_flag::WANTS_TO_WAKE,
        lithummc::plugin::player::BedrockStatusFlag::TradeInterest => {
            entity_data_flag::TRADE_INTEREST
        }
        lithummc::plugin::player::BedrockStatusFlag::DoorBreaker => entity_data_flag::DOOR_BREAKER,
        lithummc::plugin::player::BedrockStatusFlag::BreakingObstruction => {
            entity_data_flag::BREAKING_OBSTRUCTION
        }
        lithummc::plugin::player::BedrockStatusFlag::DoorOpener => entity_data_flag::DOOR_OPENER,
        lithummc::plugin::player::BedrockStatusFlag::Captain => entity_data_flag::CAPTAIN,
        lithummc::plugin::player::BedrockStatusFlag::Stunned => entity_data_flag::STUNNED,
        lithummc::plugin::player::BedrockStatusFlag::Roaring => entity_data_flag::ROARING,
        lithummc::plugin::player::BedrockStatusFlag::DelayedAttack => {
            entity_data_flag::DELAYED_ATTACK
        }
        lithummc::plugin::player::BedrockStatusFlag::AvoidingMobs => entity_data_flag::AVOIDING_MOBS,
        lithummc::plugin::player::BedrockStatusFlag::AvoidingBlock => {
            entity_data_flag::AVOIDING_BLOCK
        }
        lithummc::plugin::player::BedrockStatusFlag::FacingTargetToRangeAttack => {
            entity_data_flag::FACING_TARGET_TO_RANGE_ATTACK
        }
        lithummc::plugin::player::BedrockStatusFlag::HiddenWhenInvisible => {
            entity_data_flag::HIDDEN_WHEN_INVISIBLE
        }
        lithummc::plugin::player::BedrockStatusFlag::InUi => entity_data_flag::IN_UI,
        lithummc::plugin::player::BedrockStatusFlag::Stalking => entity_data_flag::STALKING,
        lithummc::plugin::player::BedrockStatusFlag::Emoting => entity_data_flag::EMOTING,
        lithummc::plugin::player::BedrockStatusFlag::Celebrating => entity_data_flag::CELEBRATING,
        lithummc::plugin::player::BedrockStatusFlag::Admiring => entity_data_flag::ADMIRING,
        lithummc::plugin::player::BedrockStatusFlag::CelebratingSpecial => {
            entity_data_flag::CELEBRATING_SPECIAL
        }
        lithummc::plugin::player::BedrockStatusFlag::OutOfControl => {
            entity_data_flag::OUT_OF_CONTROL
        }
        lithummc::plugin::player::BedrockStatusFlag::RamAttack => entity_data_flag::RAM_ATTACK,
        lithummc::plugin::player::BedrockStatusFlag::PlayingDead => entity_data_flag::PLAYING_DEAD,
        lithummc::plugin::player::BedrockStatusFlag::InAscendingBlock => {
            entity_data_flag::IN_ASCENDING_BLOCK
        }
        lithummc::plugin::player::BedrockStatusFlag::OverDescendingBlock => {
            entity_data_flag::OVER_DESCENDING_BLOCK
        }
        lithummc::plugin::player::BedrockStatusFlag::Croaking => entity_data_flag::CROAKING,
        lithummc::plugin::player::BedrockStatusFlag::DigestMob => entity_data_flag::DIGEST_MOB,
        lithummc::plugin::player::BedrockStatusFlag::JumpGoal => entity_data_flag::JUMP_GOAL,
        lithummc::plugin::player::BedrockStatusFlag::Emerging => entity_data_flag::EMERGING,
        lithummc::plugin::player::BedrockStatusFlag::Sniffing => entity_data_flag::SNIFFING,
        lithummc::plugin::player::BedrockStatusFlag::Digging => entity_data_flag::DIGGING,
        lithummc::plugin::player::BedrockStatusFlag::SonicBoom => entity_data_flag::SONIC_BOOM,
        lithummc::plugin::player::BedrockStatusFlag::HasDashTimeout => {
            entity_data_flag::HAS_DASH_TIMEOUT
        }
        lithummc::plugin::player::BedrockStatusFlag::PushTowardsClosestSpace => {
            entity_data_flag::PUSH_TOWARDS_CLOSEST_SPACE
        }
        lithummc::plugin::player::BedrockStatusFlag::Scenting => entity_data_flag::SCENTING,
        lithummc::plugin::player::BedrockStatusFlag::Rising => entity_data_flag::RISING,
        lithummc::plugin::player::BedrockStatusFlag::FeelingHappy => entity_data_flag::FEELING_HAPPY,
        lithummc::plugin::player::BedrockStatusFlag::Searching => entity_data_flag::SEARCHING,
        lithummc::plugin::player::BedrockStatusFlag::Crawling => entity_data_flag::CRAWLING,
        lithummc::plugin::player::BedrockStatusFlag::BodyRotationBlocked => {
            entity_data_flag::BODY_ROTATION_BLOCKED
        }
        lithummc::plugin::player::BedrockStatusFlag::RenderWhenInvisible => {
            entity_data_flag::RENDER_WHEN_INVISIBLE
        }
        lithummc::plugin::player::BedrockStatusFlag::BodyRotationAxisAligned => {
            entity_data_flag::BODY_ROTATION_AXIS_ALIGNED
        }
        lithummc::plugin::player::BedrockStatusFlag::Collidable => entity_data_flag::COLLIDABLE,
        lithummc::plugin::player::BedrockStatusFlag::WasdAirControlled => {
            entity_data_flag::WASD_AIR_CONTROLLED
        }
        lithummc::plugin::player::BedrockStatusFlag::DoesServerAuthOnlyDismount => {
            entity_data_flag::DOES_SERVER_AUTH_ONLY_DISMOUNT
        }
        lithummc::plugin::player::BedrockStatusFlag::BodyRotationAlwaysFollowsHead => {
            entity_data_flag::BODY_ROTATION_ALWAYS_FOLLOWS_HEAD
        }
        lithummc::plugin::player::BedrockStatusFlag::CanUseVerticalMovementAction => {
            entity_data_flag::CAN_USE_VERTICAL_MOVEMENT_ACTION
        }
        lithummc::plugin::player::BedrockStatusFlag::RotationLockedToVehicle => {
            entity_data_flag::ROTATION_LOCKED_TO_VEHICLE
        }
    }
}

pub(crate) fn text_component_from_resource(
    state: &PluginHostState,
    text: &Resource<lithummc::plugin::text::TextComponent>,
) -> lithummc_util::text::TextComponent {
    state
        .get(text)
        .expect("invalid text-component resource handle")
        .clone()
}

fn plugin_from_state(state: &PluginHostState) -> wasmtime::Result<Arc<WasmPlugin>> {
    state
        .plugin
        .as_ref()
        .and_then(std::sync::Weak::upgrade)
        .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))
}

pub(crate) const fn to_wit_permission_level(
    level: PermissionLvl,
) -> lithummc::plugin::permission::PermissionLevel {
    match level {
        PermissionLvl::Zero => lithummc::plugin::permission::PermissionLevel::Zero,
        PermissionLvl::One => lithummc::plugin::permission::PermissionLevel::One,
        PermissionLvl::Two => lithummc::plugin::permission::PermissionLevel::Two,
        PermissionLvl::Three => lithummc::plugin::permission::PermissionLevel::Three,
        PermissionLvl::Four => lithummc::plugin::permission::PermissionLevel::Four,
    }
}

pub(crate) const fn from_wit_permission_level(
    level: lithummc::plugin::permission::PermissionLevel,
) -> PermissionLvl {
    match level {
        lithummc::plugin::permission::PermissionLevel::Zero => PermissionLvl::Zero,
        lithummc::plugin::permission::PermissionLevel::One => PermissionLvl::One,
        lithummc::plugin::permission::PermissionLevel::Two => PermissionLvl::Two,
        lithummc::plugin::permission::PermissionLevel::Three => PermissionLvl::Three,
        lithummc::plugin::permission::PermissionLevel::Four => PermissionLvl::Four,
    }
}

pub(crate) fn parse_ban_expiry(
    expires_at_utc: Option<String>,
    duration_seconds: Option<u64>,
) -> Option<time::OffsetDateTime> {
    if let Some(dur) = duration_seconds {
        let seconds = i64::try_from(dur).unwrap_or(i64::MAX);
        return Some(time::OffsetDateTime::now_utc() + time::Duration::seconds(seconds));
    }
    if let Some(s) = expires_at_utc {
        if s.eq_ignore_ascii_case("forever") || s.is_empty() {
            return None;
        }
        if let Ok(parsed) =
            time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
        {
            return Some(parsed);
        }
        if let Ok(parsed) = time::OffsetDateTime::parse(
            &s,
            time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
            ),
        ) {
            return Some(parsed);
        }
    }
    None
}

#[allow(clippy::too_many_lines)]
const fn from_wasm_bedrock_disconnect_reason(
    reason: lithummc::plugin::player::BedrockDisconnectReason,
) -> DisconnectReason {
    match reason {
        lithummc::plugin::player::BedrockDisconnectReason::Unknown => DisconnectReason::Unknown,
        lithummc::plugin::player::BedrockDisconnectReason::CantConnectNoInternet => {
            DisconnectReason::CantConnectNoInternet
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoPermissions => {
            DisconnectReason::NoPermissions
        }
        lithummc::plugin::player::BedrockDisconnectReason::UnrecoverableError => {
            DisconnectReason::UnrecoverableError
        }
        lithummc::plugin::player::BedrockDisconnectReason::ThirdPartyBlocked => {
            DisconnectReason::ThirdPartyBlocked
        }
        lithummc::plugin::player::BedrockDisconnectReason::ThirdPartyNoInternet => {
            DisconnectReason::ThirdPartyNoInternet
        }
        lithummc::plugin::player::BedrockDisconnectReason::ThirdPartyBadIp => {
            DisconnectReason::ThirdPartyBadIP
        }
        lithummc::plugin::player::BedrockDisconnectReason::ThirdPartyNoServerOrServerLocked => {
            DisconnectReason::ThirdPartyNoServerOrServerLocked
        }
        lithummc::plugin::player::BedrockDisconnectReason::VersionMismatch => {
            DisconnectReason::VersionMismatch
        }
        lithummc::plugin::player::BedrockDisconnectReason::SkinIssue => DisconnectReason::SkinIssue,
        lithummc::plugin::player::BedrockDisconnectReason::InviteSessionNotFound => {
            DisconnectReason::InviteSessionNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::EduLevelSettingsMissing => {
            DisconnectReason::EduLevelSettingsMissing
        }
        lithummc::plugin::player::BedrockDisconnectReason::LocalServerNotFound => {
            DisconnectReason::LocalServerNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::LegacyDisconnect => {
            DisconnectReason::LegacyDisconnect
        }
        lithummc::plugin::player::BedrockDisconnectReason::UserLeaveGameAttempted => {
            DisconnectReason::UserLeaveGameAttempted
        }
        lithummc::plugin::player::BedrockDisconnectReason::PlatformLockedSkinsError => {
            DisconnectReason::PlatformLockedSkinsError
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsWorldUnassigned => {
            DisconnectReason::RealmsWorldUnassigned
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsServerCantConnect => {
            DisconnectReason::RealmsServerCantConnect
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsServerHidden => {
            DisconnectReason::RealmsServerHidden
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsServerDisabledBeta => {
            DisconnectReason::RealmsServerDisabledBeta
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsServerDisabled => {
            DisconnectReason::RealmsServerDisabled
        }
        lithummc::plugin::player::BedrockDisconnectReason::CrossPlatformDisabled => {
            DisconnectReason::CrossPlatformDisabled
        }
        lithummc::plugin::player::BedrockDisconnectReason::CantConnect => {
            DisconnectReason::CantConnect
        }
        lithummc::plugin::player::BedrockDisconnectReason::SessionNotFound => {
            DisconnectReason::SessionNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::ClientSettingsIncompatibleWithServer => {
            DisconnectReason::ClientSettingsIncompatibleWithServer
        }
        lithummc::plugin::player::BedrockDisconnectReason::ServerFull => {
            DisconnectReason::ServerFull
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidPlatformSkin => {
            DisconnectReason::InvalidPlatformSkin
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditionVersionMismatch => {
            DisconnectReason::EditionVersionMismatch
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditionMismatch => {
            DisconnectReason::EditionMismatch
        }
        lithummc::plugin::player::BedrockDisconnectReason::LevelNewerThanExeVersion => {
            DisconnectReason::LevelNewerThanExeVersion
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoFailOccurred => {
            DisconnectReason::NoFailOccurred
        }
        lithummc::plugin::player::BedrockDisconnectReason::BannedSkin => {
            DisconnectReason::BannedSkin
        }
        lithummc::plugin::player::BedrockDisconnectReason::Timeout => DisconnectReason::Timeout,
        lithummc::plugin::player::BedrockDisconnectReason::ServerNotFound => {
            DisconnectReason::ServerNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::OutdatedServer => {
            DisconnectReason::OutdatedServer
        }
        lithummc::plugin::player::BedrockDisconnectReason::OutdatedClient => {
            DisconnectReason::OutdatedClient
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoPremiumPlatform => {
            DisconnectReason::NoPremiumPlatform
        }
        lithummc::plugin::player::BedrockDisconnectReason::MultiplayerDisabled => {
            DisconnectReason::MultiplayerDisabled
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoWifi => DisconnectReason::NoWiFi,
        lithummc::plugin::player::BedrockDisconnectReason::WorldCorruption => {
            DisconnectReason::WorldCorruption
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoReason => DisconnectReason::NoReason,
        lithummc::plugin::player::BedrockDisconnectReason::Disconnected => {
            DisconnectReason::Disconnected
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidPlayer => {
            DisconnectReason::InvalidPlayer
        }
        lithummc::plugin::player::BedrockDisconnectReason::LoggedInOtherLocation => {
            DisconnectReason::LoggedInOtherLocation
        }
        lithummc::plugin::player::BedrockDisconnectReason::ServerIdConflict => {
            DisconnectReason::ServerIdConflict
        }
        lithummc::plugin::player::BedrockDisconnectReason::NotAllowed => {
            DisconnectReason::NotAllowed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NotAuthenticated => {
            DisconnectReason::NotAuthenticated
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidTenant => {
            DisconnectReason::InvalidTenant
        }
        lithummc::plugin::player::BedrockDisconnectReason::UnknownPacket => {
            DisconnectReason::UnknownPacket
        }
        lithummc::plugin::player::BedrockDisconnectReason::UnexpectedPacket => {
            DisconnectReason::UnexpectedPacket
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidCommandRequestPacket => {
            DisconnectReason::InvalidCommandRequestPacket
        }
        lithummc::plugin::player::BedrockDisconnectReason::HostSuspended => {
            DisconnectReason::HostSuspended
        }
        lithummc::plugin::player::BedrockDisconnectReason::LoginPacketNoRequest => {
            DisconnectReason::LoginPacketNoRequest
        }
        lithummc::plugin::player::BedrockDisconnectReason::LoginPacketNoCert => {
            DisconnectReason::LoginPacketNoCert
        }
        lithummc::plugin::player::BedrockDisconnectReason::MissingClient => {
            DisconnectReason::MissingClient
        }
        lithummc::plugin::player::BedrockDisconnectReason::Kicked => DisconnectReason::Kicked,
        lithummc::plugin::player::BedrockDisconnectReason::KickedForExploit => {
            DisconnectReason::KickedForExploit
        }
        lithummc::plugin::player::BedrockDisconnectReason::KickedForIdle => {
            DisconnectReason::KickedForIdle
        }
        lithummc::plugin::player::BedrockDisconnectReason::ResourcePackProblem => {
            DisconnectReason::ResourcePackProblem
        }
        lithummc::plugin::player::BedrockDisconnectReason::IncompatiblePack => {
            DisconnectReason::IncompatiblePack
        }
        lithummc::plugin::player::BedrockDisconnectReason::OutOfStorage => {
            DisconnectReason::OutOfStorage
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidLevel => {
            DisconnectReason::InvalidLevel
        }
        lithummc::plugin::player::BedrockDisconnectReason::DisconnectPacket => {
            DisconnectReason::DisconnectPacket
        }
        lithummc::plugin::player::BedrockDisconnectReason::BlockMismatch => {
            DisconnectReason::BlockMismatch
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidHeights => {
            DisconnectReason::InvalidHeights
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidWidths => {
            DisconnectReason::InvalidWidths
        }
        lithummc::plugin::player::BedrockDisconnectReason::ConnectionLost => {
            DisconnectReason::ConnectionLost
        }
        lithummc::plugin::player::BedrockDisconnectReason::ZombieConnection => {
            DisconnectReason::ZombieConnection
        }
        lithummc::plugin::player::BedrockDisconnectReason::Shutdown => DisconnectReason::Shutdown,
        lithummc::plugin::player::BedrockDisconnectReason::ReasonNotSet => {
            DisconnectReason::ReasonNotSet
        }
        lithummc::plugin::player::BedrockDisconnectReason::LoadingStateTimeout => {
            DisconnectReason::LoadingStateTimeout
        }
        lithummc::plugin::player::BedrockDisconnectReason::ResourcePackLoadingFailed => {
            DisconnectReason::ResourcePackLoadingFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::SearchingForSessionLoadingScreenFailed => {
            DisconnectReason::SearchingForSessionLoadingScreenFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetProtocolVersion => {
            DisconnectReason::NetherNetProtocolVersion
        }
        lithummc::plugin::player::BedrockDisconnectReason::SubsystemStatusError => {
            DisconnectReason::SubsystemStatusError
        }
        lithummc::plugin::player::BedrockDisconnectReason::EmptyAuthFromDiscovery => {
            DisconnectReason::EmptyAuthFromDiscovery
        }
        lithummc::plugin::player::BedrockDisconnectReason::EmptyUrlFromDiscovery => {
            DisconnectReason::EmptyUrlFromDiscovery
        }
        lithummc::plugin::player::BedrockDisconnectReason::ExpiredAuthFromDiscovery => {
            DisconnectReason::ExpiredAuthFromDiscovery
        }
        lithummc::plugin::player::BedrockDisconnectReason::UnknownSignalServiceSignInFailure => {
            DisconnectReason::UnknownSignalServiceSignInFailure
        }
        lithummc::plugin::player::BedrockDisconnectReason::XblJoinLobbyFailure => {
            DisconnectReason::XBLJoinLobbyFailure
        }
        lithummc::plugin::player::BedrockDisconnectReason::UnspecifiedClientInstanceDisconnection => {
            DisconnectReason::UnspecifiedClientInstanceDisconnection
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSessionNotFound => {
            DisconnectReason::NetherNetSessionNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetCreatePeerConnection => {
            DisconnectReason::NetherNetCreatePeerConnection
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetIce => {
            DisconnectReason::NetherNetICE
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetConnectRequest => {
            DisconnectReason::NetherNetConnectRequest
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetConnectResponse => {
            DisconnectReason::NetherNetConnectResponse
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetNegotiationTimeout => {
            DisconnectReason::NetherNetNegotiationTimeout
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetInactivityTimeout => {
            DisconnectReason::NetherNetInactivityTimeout
        }
        lithummc::plugin::player::BedrockDisconnectReason::StaleConnectionBeingReplaced => {
            DisconnectReason::StaleConnectionBeingReplaced
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsSessionNotFound => {
            DisconnectReason::RealmsSessionNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::BadPacket => DisconnectReason::BadPacket,
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetFailedToCreateOffer => {
            DisconnectReason::NetherNetFailedToCreateOffer
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetFailedToCreateAnswer => {
            DisconnectReason::NetherNetFailedToCreateAnswer
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetFailedToSetLocalDescription => {
            DisconnectReason::NetherNetFailedToSetLocalDescription
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetFailedToSetRemoteDescription => {
            DisconnectReason::NetherNetFailedToSetRemoteDescription
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetNegotiationTimeoutWaitingForResponse => {
            DisconnectReason::NetherNetNegotiationTimeoutWaitingForResponse
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetNegotiationTimeoutWaitingForAccept => {
            DisconnectReason::NetherNetNegotiationTimeoutWaitingForAccept
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetIncomingConnectionIgnored => {
            DisconnectReason::NetherNetIncomingConnectionIgnored
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingParsingFailure => {
            DisconnectReason::NetherNetSignalingParsingFailure
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingUnknownError => {
            DisconnectReason::NetherNetSignalingUnknownError
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingUnicastDeliveryFailed => {
            DisconnectReason::NetherNetSignalingUnicastDeliveryFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingBroadcastDeliveryFailed => {
            DisconnectReason::NetherNetSignalingBroadcastDeliveryFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingGenericDeliveryFailed => {
            DisconnectReason::NetherNetSignalingGenericDeliveryFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorMismatchEditorWorld => {
            DisconnectReason::EditorMismatchEditorWorld
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorMismatchVanillaWorld => {
            DisconnectReason::EditorMismatchVanillaWorld
        }
        lithummc::plugin::player::BedrockDisconnectReason::WorldTransferNotPrimaryClient => {
            DisconnectReason::WorldTransferNotPrimaryClient
        }
        lithummc::plugin::player::BedrockDisconnectReason::RequestServerShutdown => {
            DisconnectReason::RequestServerShutdown
        }
        lithummc::plugin::player::BedrockDisconnectReason::ClientGameSetupCancelled => {
            DisconnectReason::ClientGameSetupCancelled
        }
        lithummc::plugin::player::BedrockDisconnectReason::ClientGameSetupFailed => {
            DisconnectReason::ClientGameSetupFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NoVenue => DisconnectReason::NoVenue,
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetSignalingSigninFailed => {
            DisconnectReason::NetherNetSignalingSigninFailed
        }
        lithummc::plugin::player::BedrockDisconnectReason::SessionAccessDenied => {
            DisconnectReason::SessionAccessDenied
        }
        lithummc::plugin::player::BedrockDisconnectReason::ServiceSigninIssue => {
            DisconnectReason::ServiceSigninIssue
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetNoSignalingChannel => {
            DisconnectReason::NetherNetNoSignalingChannel
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetNotLoggedIn => {
            DisconnectReason::NetherNetNotLoggedIn
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetClientSignalingError => {
            DisconnectReason::NetherNetClientSignalingError
        }
        lithummc::plugin::player::BedrockDisconnectReason::SubClientLoginDisabled => {
            DisconnectReason::SubClientLoginDisabled
        }
        lithummc::plugin::player::BedrockDisconnectReason::DeepLinkTryingToOpenDemoWorldWhileSignedIn => {
            DisconnectReason::DeepLinkTryingToOpenDemoWorldWhileSignedIn
        }
        lithummc::plugin::player::BedrockDisconnectReason::AsyncJoinTaskDenied => {
            DisconnectReason::AsyncJoinTaskDenied
        }
        lithummc::plugin::player::BedrockDisconnectReason::RealmsTimelineRequired => {
            DisconnectReason::RealmsTimelineRequired
        }
        lithummc::plugin::player::BedrockDisconnectReason::GuestWithoutHost => {
            DisconnectReason::GuestWithoutHost
        }
        lithummc::plugin::player::BedrockDisconnectReason::FailedToJoinExperience => {
            DisconnectReason::FailedToJoinExperience
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetDataChannelClosed => {
            DisconnectReason::NetherNetDataChannelClosed
        }
        lithummc::plugin::player::BedrockDisconnectReason::DiscoveryEnvironmentMismatch => {
            DisconnectReason::DiscoveryEnvironmentMismatch
        }
        lithummc::plugin::player::BedrockDisconnectReason::HostWithoutKeys => {
            DisconnectReason::HostWithoutKeys
        }
        lithummc::plugin::player::BedrockDisconnectReason::HostSignedOut => {
            DisconnectReason::HostSignedOut
        }
        lithummc::plugin::player::BedrockDisconnectReason::ScriptWatchdogException => {
            DisconnectReason::ScriptWatchdogException
        }
        lithummc::plugin::player::BedrockDisconnectReason::ScriptMemoryLimitExceeded => {
            DisconnectReason::ScriptMemoryLimitExceeded
        }
        lithummc::plugin::player::BedrockDisconnectReason::StorageLowDuringGameplay => {
            DisconnectReason::StorageLowDuringGameplay
        }
        lithummc::plugin::player::BedrockDisconnectReason::StorageFullDuringGameplay => {
            DisconnectReason::StorageFullDuringGameplay
        }
        lithummc::plugin::player::BedrockDisconnectReason::LevelStorageCorruption => {
            DisconnectReason::LevelStorageCorruption
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditionMismatchVanillaToEdu => {
            DisconnectReason::EditionMismatchVanillaToEdu
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditionMismatchEduToVanilla => {
            DisconnectReason::EditionMismatchEduToVanilla
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorMismatchEditorToVanilla => {
            DisconnectReason::EditorMismatchEditorToVanilla
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorMismatchVanillaToEditor => {
            DisconnectReason::EditorMismatchVanillaToEditor
        }
        lithummc::plugin::player::BedrockDisconnectReason::DenyListed => {
            DisconnectReason::DenyListed
        }
        lithummc::plugin::player::BedrockDisconnectReason::NonceMissing => {
            DisconnectReason::NonceMissing
        }
        lithummc::plugin::player::BedrockDisconnectReason::NonceNotFound => {
            DisconnectReason::NonceNotFound
        }
        lithummc::plugin::player::BedrockDisconnectReason::NonceExpired => {
            DisconnectReason::NonceExpired
        }
        lithummc::plugin::player::BedrockDisconnectReason::NonceNotValid => {
            DisconnectReason::NonceNotValid
        }
        lithummc::plugin::player::BedrockDisconnectReason::HostDisconnected => {
            DisconnectReason::HostDisconnected
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorJoinIntentPolicyFailure => {
            DisconnectReason::EditorJoinIntentPolicyFailure
        }
        lithummc::plugin::player::BedrockDisconnectReason::NetherNetIdentityNotAllowed => {
            DisconnectReason::NetherNetIdentityNotAllowed
        }
        lithummc::plugin::player::BedrockDisconnectReason::InvalidName => {
            DisconnectReason::InvalidName
        }
        lithummc::plugin::player::BedrockDisconnectReason::ExpiredToken => {
            DisconnectReason::ExpiredToken
        }
        lithummc::plugin::player::BedrockDisconnectReason::HostAcceptsNoTypeOfAuth => {
            DisconnectReason::HostAcceptsNoTypeOfAuth
        }
        lithummc::plugin::player::BedrockDisconnectReason::NotAuthenticatedFastFail => {
            DisconnectReason::NotAuthenticatedFastFail
        }
        lithummc::plugin::player::BedrockDisconnectReason::EditorNotAllowed => {
            DisconnectReason::EditorNotAllowed
        }
    }
}

impl lithummc::plugin::player::Host for PluginHostState {
    fn get_world_players(
        &mut self,
        world: Resource<lithummc::plugin::world::World>,
    ) -> wasmtime::Result<Vec<Resource<lithummc::plugin::player::Player>>> {
        let world = self.take(world)?;

        let mut players = Vec::new();
        for player in world.players.load().iter() {
            players.push(self.add(player.clone())?);
        }

        Ok(players)
    }
}
use crate::events::from_wasm_hand;
use lithummc_inventory::generic_container_screen_handler::GenericContainerScreenHandler;
use lithummc_inventory::player::ender_chest_inventory::EnderChestInventory;
use lithummc_inventory::{Clearable, Inventory};
use lithummc_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use lithummc_protocol::java::client::play::CSetContainerSlot;

use crate::lithummc::plugin::item_stack::ItemStack as WitHostItemStack;

impl lithummc::plugin::player::HostPlayer for PluginHostState {
    async fn set_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: lithummc::plugin::common::Hand,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = stack {
            self.take(stack_res)?.lock().await.clone()
        } else {
            lithummc_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = self.get(&player)?;

        let hand = from_wasm_hand(hand);
        let slot = match hand {
            lithummc_util::Hand::Right => player.inventory().get_selected_slot() as usize,
            lithummc_util::Hand::Left => PlayerInventory::OFF_HAND_SLOT,
        };

        player.inventory().set_stack(slot, stack.clone());
        player.sync_hand_slot(slot, stack);

        Ok(())
    }

    async fn set_inventory_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = stack {
            self.take(stack_res)?.lock().await.clone()
        } else {
            lithummc_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = self.get(&player)?;

        player.inventory().set_stack(slot as usize, stack.clone());

        player.sync_hand_slot(slot as usize, stack);

        Ok(())
    }

    fn get_inventory(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<crate::lithummc::plugin::inventory::PlayerInventory>> {
        self.add(self.get(&player)?.clone())
    }

    fn get_ender_chest(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<crate::lithummc::plugin::inventory::Inventory>> {
        self.add(
            lithummc_wasm_host_common::state::InventoryProvider::PlayerEnderChest(
                self.get(&player)?.clone(),
            ),
        )
    }

    fn get_inventory_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&player)?;
        let stack = player.inventory().get_stack(slot as usize);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(stack)))?))
        }
    }

    fn get_ender_chest_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&player)?;
        let ec = player.ender_chest_inventory();
        let stack = ec.get_stack(slot as usize);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(stack)))?))
        }
    }

    async fn set_ender_chest_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = stack {
            self.take(stack_res)?.lock().await.clone()
        } else {
            lithummc_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = self.get(&player)?;

        let ec = player.ender_chest_inventory();
        ec.set_stack(slot as usize, stack.clone());

        // If the player currently has their ender chest screen open, sync slot
        let sync_id = {
            let screen_handler_arc = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let handler = screen_handler_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(generic) = handler
                .as_any()
                .downcast_ref::<GenericContainerScreenHandler>()
                && generic.inventory.as_any().is::<EnderChestInventory>()
            {
                Some(handler.sync_id())
            } else {
                None
            }
        };
        if let Some(sync_id) = sync_id {
            let stack_serializer = ItemStackSerializer::from(stack);
            let packet = CSetContainerSlot::new(sync_id as i8, 0, slot as i16, &stack_serializer);
            player.send_client_packet(&packet).await;
        }

        Ok(())
    }

    async fn clear_ender_chest(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let ec = player.ender_chest_inventory();
        ec.clear();

        // If the player currently has their ender chest screen open, sync all slots
        let sync_id = {
            let screen_handler_arc = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let handler = screen_handler_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(generic) = handler
                .as_any()
                .downcast_ref::<GenericContainerScreenHandler>()
                && generic.inventory.as_any().is::<EnderChestInventory>()
            {
                Some(handler.sync_id())
            } else {
                None
            }
        };
        if let Some(sync_id) = sync_id {
            let empty_serializer =
                ItemStackSerializer::from(lithummc_data::item_stack::ItemStack::EMPTY.clone());
            for slot in 0..27 {
                let packet =
                    CSetContainerSlot::new(sync_id as i8, 0, slot as i16, &empty_serializer);
                player.send_client_packet(&packet).await;
            }
        }

        Ok(())
    }

    fn get_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: lithummc::plugin::common::Hand,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&player)?;
        let hand = from_wasm_hand(hand);
        let stack = player.inventory().get_stack_in_hand(hand);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(tokio::sync::Mutex::new(stack)))?))
        }
    }

    fn as_entity(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<lithummc::plugin::world::Entity>> {
        self.add(self.get(&player)?.clone() as _)
    }

    fn get_id(&mut self, player: Resource<Player>) -> wasmtime::Result<Uuid> {
        let player = self.get(&player)?;
        Ok(Uuid::to_wit(&player.gameprofile.id))
    }

    fn get_name(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = self.get(&player)?;
        Ok(player.gameprofile.name.clone())
    }

    fn get_position(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<lithummc::plugin::common::Position> {
        let player = self.get(&player)?;
        Ok(to_wasm_position(player.position()))
    }

    fn get_yaw(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.get_entity().yaw.load())
    }

    fn get_pitch(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.get_entity().pitch.load())
    }

    fn get_world(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<lithummc::plugin::world::World>> {
        let player = self.get(&player)?;
        let world = player.world();
        self.add(world)
            .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
    }

    fn get_gamemode(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<lithummc::plugin::common::GameMode> {
        let player = self.get(&player)?;
        Ok(to_wasm_game_mode(player.gamemode.load()))
    }

    fn get_locale(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = self.get(&player)?;
        Ok(player.config.load().locale.clone())
    }

    fn get_ping(&mut self, player: Resource<Player>) -> wasmtime::Result<u32> {
        let player = self.get(&player)?;
        Ok(player.ping.load(Ordering::Relaxed))
    }

    fn get_permission_level(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<lithummc::plugin::permission::PermissionLevel> {
        let player = self.get(&player)?;
        Ok(to_wit_permission_level(player.permission_lvl.load()))
    }

    fn set_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let server = self.server.as_ref().expect("server not available");

        server
            .permission_manager
            .set_permission(player.gameprofile.id, node, value);

        Ok(())
    }

    fn unset_permission(&mut self, player: Resource<Player>, node: String) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let server = self.server.as_ref().expect("server not available");

        server
            .permission_manager
            .unset_permission(&player.gameprofile.id, &node);

        Ok(())
    }

    fn has_permission_set(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<Option<bool>> {
        let player = self.get(&player)?;
        let server = self.server.as_ref().expect("server not available");

        Ok(server
            .permission_manager
            .has_permission_set(&player.gameprofile.id, &node))
    }

    fn get_display_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<lithummc::plugin::text::TextComponent>> {
        let player = self.get(&player)?;
        let display_name = player.get_display_name();
        self.add(display_name)
            .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
    }

    fn set_display_name(
        &mut self,
        player: Resource<Player>,
        display_name: Resource<lithummc::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let display_name = self.take(display_name)?;
        let player = self.get(&player)?;
        player.set_display_name(Some(display_name));
        Ok(())
    }

    fn get_tab_list_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<lithummc::plugin::text::TextComponent>>> {
        let player = self.get(&player)?;
        let tab_list_name = player.get_tab_list_name();
        tab_list_name.map_or_else(
            || Ok(None),
            |name| {
                self.add(name)
                    .map(Some)
                    .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
            },
        )
    }

    fn set_tab_list_name(
        &mut self,
        player: Resource<Player>,
        name: Option<Resource<lithummc::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let name = name.map(|n| self.take(n)).transpose()?;
        let player = self.get(&player)?;
        player.set_tab_list_name(name);
        Ok(())
    }

    fn send_system_message(
        &mut self,
        player: Resource<Player>,
        text: Resource<lithummc::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        let player = self.get(&player)?;
        player.send_system_message_raw(&component, overlay);
        Ok(())
    }

    async fn delete_message_by_signature(
        &mut self,
        player: Resource<Player>,
        signature: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        if let Some(client) = player.client.java() {
            let packet =
                lithummc_protocol::java::client::play::CDeleteChat::from_signature(&signature);
            client.send_packet(&packet).await;
        }
        Ok(())
    }

    async fn delete_message_by_id(
        &mut self,
        player: Resource<Player>,
        signature_id: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        if let Some(client) = player.client.java() {
            let packet =
                lithummc_protocol::java::client::play::CDeleteChat::from_cache_id(signature_id);
            client.send_packet(&packet).await;
        }
        Ok(())
    }

    fn set_camera(
        &mut self,
        player: Resource<Player>,
        entity: Option<Resource<crate::lithummc::plugin::world::Entity>>,
    ) -> wasmtime::Result<()> {
        let entity = entity.map(|r| self.take(r)).transpose()?;
        let player = self.get(&player)?;
        if let Some(target) = entity {
            player.set_camera_entity_id(target.get_entity().entity_id);
        } else {
            player.reset_camera();
        }
        Ok(())
    }

    fn set_camera_entity_id(
        &mut self,
        player: Resource<Player>,
        entity_id: u32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_camera_entity_id(entity_id as i32);
        Ok(())
    }

    fn get_camera_entity_id(&mut self, player: Resource<Player>) -> wasmtime::Result<u32> {
        let player = self.get(&player)?;
        Ok(player.get_camera_entity_id() as u32)
    }

    fn reset_camera(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.reset_camera();
        Ok(())
    }

    fn play_sound(
        &mut self,
        player: Resource<Player>,
        sound: lithummc::plugin::sounds::Sound,
        category: lithummc::plugin::sounds::SoundCategory,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let sound_name = format!("{sound:?}").to_lowercase().replace('_', ".");
        let sound_data = lithummc_data::sound::Sound::from_name(&sound_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown sound: {sound_name}")))?;
        let internal_category = super::world::from_wit_sound_category(category);
        let pos = player.position();
        player.play_sound(
            sound_data as u16,
            internal_category,
            &pos,
            volume,
            pitch,
            rand::random::<i64>(),
        );
        Ok(())
    }

    fn play_sound_at(
        &mut self,
        player: Resource<Player>,
        pos: lithummc::plugin::common::Position,
        sound: lithummc::plugin::sounds::Sound,
        category: lithummc::plugin::sounds::SoundCategory,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let sound_name = format!("{sound:?}").to_lowercase().replace('_', ".");
        let sound_data = lithummc_data::sound::Sound::from_name(&sound_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown sound: {sound_name}")))?;
        let internal_category = super::world::from_wit_sound_category(category);
        player.play_sound(
            sound_data as u16,
            internal_category,
            &lithummc_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            volume,
            pitch,
            rand::random::<i64>(),
        );
        Ok(())
    }

    fn stop_sound(
        &mut self,
        player: Resource<Player>,
        sound: Option<lithummc::plugin::sounds::Sound>,
        category: Option<lithummc::plugin::sounds::SoundCategory>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let sound_rl = sound.and_then(|s| {
            let sound_name = format!("{s:?}").to_lowercase().replace('_', ".");
            lithummc_data::sound::Sound::from_name(&sound_name).map(|s| s.to_name().into())
        });
        let cat = category.map(super::world::from_wit_sound_category);
        player.stop_sound(sound_rl, cat);
        Ok(())
    }

    fn play_custom_sound(
        &mut self,
        player: Resource<Player>,
        sound_name: String,
        category: lithummc::plugin::sounds::SoundCategory,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let internal_category = super::world::from_wit_sound_category(category);
        let pos = player.position();
        player.play_custom_sound(&sound_name, internal_category, &pos, volume, pitch);
        Ok(())
    }

    fn play_custom_sound_at(
        &mut self,
        player: Resource<Player>,
        pos: lithummc::plugin::common::Position,
        sound_name: String,
        category: lithummc::plugin::sounds::SoundCategory,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let internal_category = super::world::from_wit_sound_category(category);
        player.play_custom_sound(
            &sound_name,
            internal_category,
            &lithummc_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            volume,
            pitch,
        );
        Ok(())
    }

    fn stop_custom_sound(
        &mut self,
        player: Resource<Player>,
        sound_name: Option<String>,
        category: Option<lithummc::plugin::sounds::SoundCategory>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let sound_rl = sound_name
            .as_deref()
            .map(lithummc_util::resource_location::ResourceLocation::from);
        let cat = category.map(super::world::from_wit_sound_category);
        player.stop_sound(sound_rl, cat);
        Ok(())
    }

    fn spawn_particles(
        &mut self,
        player: Resource<Player>,
        particle: lithummc::plugin::particles::Particle,
        pos: lithummc::plugin::common::Position,
        count: u32,
        offset: lithummc::plugin::common::Position,
        max_speed: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let particle_data =
            lithummc_data::particle::Particle::from_id(particle as u16).ok_or_else(|| {
                wasmtime::Error::msg(format!("Unknown particle ID: {}", particle as u16))
            })?;
        player.spawn_particles(
            particle_data,
            lithummc_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            count,
            lithummc_util::math::vector3::Vector3::new(
                offset.0 as f32,
                offset.1 as f32,
                offset.2 as f32,
            ),
            max_speed,
        );
        Ok(())
    }

    fn send_block_change(
        &mut self,
        player: Resource<Player>,
        pos: lithummc::plugin::common::BlockPos,
        block_id: u16,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.send_block_change(
            lithummc_util::math::position::BlockPos(lithummc_util::math::vector3::Vector3::new(
                pos.x, pos.y, pos.z,
            )),
            block_id,
        );
        Ok(())
    }

    fn reset_block_change(
        &mut self,
        player: Resource<Player>,
        pos: lithummc::plugin::common::BlockPos,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.reset_block_change(lithummc_util::math::position::BlockPos(
            lithummc_util::math::vector3::Vector3::new(pos.x, pos.y, pos.z),
        ));
        Ok(())
    }

    fn send_hurt_animation(&mut self, player: Resource<Player>, yaw: f32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.send_hurt_animation(yaw);
        Ok(())
    }

    fn open_book(
        &mut self,
        player: Resource<Player>,
        hand: lithummc::plugin::common::Hand,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let hand = match hand {
            lithummc::plugin::common::Hand::Right => lithummc_util::Hand::Right,
            lithummc::plugin::common::Hand::Left => lithummc_util::Hand::Left,
        };
        player.open_book(hand);
        Ok(())
    }

    fn open_sign_editor(
        &mut self,
        player: Resource<Player>,
        pos: lithummc::plugin::common::BlockPos,
        is_front_text: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.open_sign_editor(
            lithummc_util::math::position::BlockPos(lithummc_util::math::vector3::Vector3::new(
                pos.x, pos.y, pos.z,
            )),
            is_front_text,
        );
        Ok(())
    }

    fn set_velocity(
        &mut self,
        player: Resource<Player>,
        velocity: lithummc::plugin::common::Position,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_velocity(lithummc_util::math::vector3::Vector3::new(
            velocity.0, velocity.1, velocity.2,
        ));
        Ok(())
    }

    fn apply_knockback(
        &mut self,
        player: Resource<Player>,
        strength: f64,
        x: f64,
        z: f64,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.apply_knockback(strength, x, z);
        Ok(())
    }

    fn set_movement_locked(
        &mut self,
        player: Resource<Player>,
        locked: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_movement_locked(locked);
        Ok(())
    }

    fn is_movement_locked(&mut self, player: Resource<Player>) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        Ok(player.is_movement_locked())
    }

    fn set_freeze_ticks(&mut self, player: Resource<Player>, ticks: i32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_freeze_ticks(ticks);
        Ok(())
    }

    fn get_freeze_ticks(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = self.get(&player)?;
        Ok(player.get_freeze_ticks())
    }

    fn set_server_links(
        &mut self,
        player: Resource<Player>,
        links: Vec<lithummc::plugin::player::ServerLink>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();
        let mut converted = Vec::new();
        for link in links {
            converted.push(from_wit_server_link(self, link)?);
        }
        let protocol_links: Vec<lithummc_protocol::Link<'_>> = converted
            .iter()
            .map(|(label, url)| lithummc_protocol::Link::new(label.clone(), url))
            .collect();
        player.set_server_links(&protocol_links);
        Ok(())
    }

    fn remove_effect(
        &mut self,
        player: Resource<Player>,
        effect: lithummc::plugin::status_effect::StatusEffectType,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        if let Some(status_effect) =
            lithummc_data::effect::StatusEffect::from_name(effect_type.to_name())
        {
            player.remove_effect(status_effect);
        }
        Ok(())
    }

    fn clear_effects(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.remove_all_effects();
        Ok(())
    }

    fn has_effect(
        &mut self,
        player: Resource<Player>,
        effect: lithummc::plugin::status_effect::StatusEffectType,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        Ok(
            lithummc_data::effect::StatusEffect::from_name(effect_type.to_name())
                .is_some_and(|status_effect| player.has_effect(status_effect)),
        )
    }

    fn get_effect(
        &mut self,
        player: Resource<Player>,
        effect: lithummc::plugin::status_effect::StatusEffectType,
    ) -> wasmtime::Result<Option<lithummc::plugin::status_effect::StatusEffectInstance>> {
        let player = self.get(&player)?;
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect);
        if let Some(status_effect) =
            lithummc_data::effect::StatusEffect::from_name(effect_type.to_name())
            && let Some(eff) = player.get_effect(status_effect)
        {
            return Ok(super::status_effect::to_wasm_status_effect_instance(&eff));
        }
        Ok(None)
    }

    fn get_active_effects(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Vec<lithummc::plugin::status_effect::StatusEffectInstance>> {
        let player = self.get(&player)?;
        let effects = player.get_active_effects();
        let mut list = Vec::with_capacity(effects.len());
        for eff in &effects {
            if let Some(instance) = super::status_effect::to_wasm_status_effect_instance(eff) {
                list.push(instance);
            }
        }
        Ok(list)
    }

    fn get_statistic(
        &mut self,
        player: Resource<Player>,
        category: WitStatisticCategory,
        stat_id: i32,
    ) -> wasmtime::Result<i32> {
        let player = self.get(&player)?;
        Ok(player.get_stat(from_wit_statistic_category(category), stat_id))
    }

    fn set_statistic(
        &mut self,
        player: Resource<Player>,
        category: WitStatisticCategory,
        stat_id: i32,
        value: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_stat(from_wit_statistic_category(category), stat_id, value);
        Ok(())
    }

    fn increment_statistic(
        &mut self,
        player: Resource<Player>,
        category: WitStatisticCategory,
        stat_id: i32,
        amount: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.increment_stat(from_wit_statistic_category(category), stat_id, amount);
        Ok(())
    }

    fn get_custom_statistic(
        &mut self,
        player: Resource<Player>,
        stat: WitCustomStatistic,
    ) -> wasmtime::Result<i32> {
        let player = self.get(&player)?;
        Ok(player.get_custom_stat(from_wit_custom_statistic(stat)))
    }

    fn set_custom_statistic(
        &mut self,
        player: Resource<Player>,
        stat: WitCustomStatistic,
        value: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_custom_stat(from_wit_custom_statistic(stat), value);
        Ok(())
    }

    fn increment_custom_statistic(
        &mut self,
        player: Resource<Player>,
        stat: WitCustomStatistic,
        amount: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.increment_custom_stat(from_wit_custom_statistic(stat), amount);
        Ok(())
    }

    fn send_stats(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.send_stats();
        Ok(())
    }

    fn get_team(&mut self, player: Resource<Player>) -> wasmtime::Result<Option<String>> {
        let player = self.get(&player)?;
        let team = player.get_team();
        Ok(team.map(|t| t.name))
    }

    fn start_cooldown(
        &mut self,
        player: Resource<Player>,
        group: String,
        duration_ticks: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.start_cooldown(group, duration_ticks);
        Ok(())
    }

    fn get_cooldown(&mut self, player: Resource<Player>, group: String) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.get_cooldown(&group))
    }

    fn is_on_cooldown(
        &mut self,
        player: Resource<Player>,
        group: String,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        Ok(player.is_on_cooldown(&group))
    }

    fn set_allow_flight(
        &mut self,
        player: Resource<Player>,
        allowed: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_allow_flight(allowed);
        Ok(())
    }

    fn set_fly_speed(&mut self, player: Resource<Player>, speed: f32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_fly_speed(speed);
        Ok(())
    }

    fn set_walk_speed(&mut self, player: Resource<Player>, speed: f32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_walk_speed(speed);
        Ok(())
    }

    fn set_invulnerable(
        &mut self,
        player: Resource<Player>,
        invulnerable: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_invulnerable(invulnerable);
        Ok(())
    }

    fn set_player_time(
        &mut self,
        player: Resource<Player>,
        time: u64,
        relative: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_player_time(time, relative);
        Ok(())
    }

    fn reset_player_time(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.reset_player_time();
        Ok(())
    }

    fn get_player_time(&mut self, player: Resource<Player>) -> wasmtime::Result<Option<u64>> {
        let player = self.get(&player)?;
        Ok(player.get_player_time())
    }

    fn is_player_time_relative(&mut self, player: Resource<Player>) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        Ok(player.is_player_time_relative())
    }

    fn set_player_weather(
        &mut self,
        player: Resource<Player>,
        weather: crate::lithummc::plugin::player::PlayerWeather,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let w = match weather {
            crate::lithummc::plugin::player::PlayerWeather::Clear => {
                lithummc_core::entity::player::PlayerWeather::Clear
            }
            crate::lithummc::plugin::player::PlayerWeather::Downfall => {
                lithummc_core::entity::player::PlayerWeather::Downfall
            }
        };
        player.set_player_weather(w);
        Ok(())
    }

    fn reset_player_weather(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.reset_player_weather();
        Ok(())
    }

    fn get_player_weather(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<crate::lithummc::plugin::player::PlayerWeather>> {
        let player = self.get(&player)?;
        Ok(player.get_player_weather().map(|w| match w {
            lithummc_core::entity::player::PlayerWeather::Clear => {
                crate::lithummc::plugin::player::PlayerWeather::Clear
            }
            lithummc_core::entity::player::PlayerWeather::Downfall => {
                crate::lithummc::plugin::player::PlayerWeather::Downfall
            }
        }))
    }

    fn set_compass_target(
        &mut self,
        player: Resource<Player>,
        pos: crate::lithummc::plugin::common::Position,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let pos_vec = from_wasm_position(pos);
        let block_pos = lithummc_util::math::position::BlockPos::new(
            pos_vec.x as i32,
            pos_vec.y as i32,
            pos_vec.z as i32,
        );
        player.set_compass_target(block_pos);
        Ok(())
    }

    fn get_compass_target(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<crate::lithummc::plugin::common::Position> {
        let player = self.get(&player)?;
        let target = player
            .get_compass_target()
            .unwrap_or(lithummc_util::math::position::BlockPos::new(0, 0, 0));
        let vec3 = lithummc_util::math::vector3::Vector3::new(
            f64::from(target.0.x),
            f64::from(target.0.y),
            f64::from(target.0.z),
        );
        Ok(to_wasm_position(vec3))
    }

    fn set_respawn_location(
        &mut self,
        player: Resource<Player>,
        pos: crate::lithummc::plugin::common::Position,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let pos_vec = from_wasm_position(pos);
        let block_pos = lithummc_util::math::position::BlockPos::new(
            pos_vec.x as i32,
            pos_vec.y as i32,
            pos_vec.z as i32,
        );
        player.set_respawn_location(block_pos);
        Ok(())
    }

    fn get_respawn_location(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<crate::lithummc::plugin::common::Position>> {
        let player = self.get(&player)?;
        Ok(player.get_respawn_location().map(|p| {
            let vec3 = lithummc_util::math::vector3::Vector3::new(
                f64::from(p.0.x),
                f64::from(p.0.y),
                f64::from(p.0.z),
            );
            to_wasm_position(vec3)
        }))
    }

    fn hide_player(
        &mut self,
        player: Resource<Player>,
        other: Resource<Player>,
    ) -> wasmtime::Result<()> {
        let other = self.take(other)?;
        let player = self.get(&player)?;
        player.hide_player(other.gameprofile.id);
        Ok(())
    }

    fn show_player(
        &mut self,
        player: Resource<Player>,
        other: Resource<Player>,
    ) -> wasmtime::Result<()> {
        let other = self.take(other)?;
        let player = self.get(&player)?;
        player.show_player(other.gameprofile.id);
        Ok(())
    }

    fn can_see(
        &mut self,
        player: Resource<Player>,
        other: Resource<Player>,
    ) -> wasmtime::Result<bool> {
        let other = self.take(other)?;
        let player = self.get(&player)?;
        Ok(player.can_see(&other.gameprofile.id))
    }

    fn can_see_player(
        &mut self,
        player: Resource<Player>,
        other: Resource<Player>,
    ) -> wasmtime::Result<bool> {
        let other = self.take(other)?;
        let player = self.get(&player)?;
        Ok(player.can_see(&other.gameprofile.id))
    }

    fn set_tab_list_ping(
        &mut self,
        player: Resource<Player>,
        latency_ms: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_tab_list_ping(latency_ms);
        Ok(())
    }

    fn set_item_cooldown(
        &mut self,
        player: Resource<Player>,
        item_id: String,
        ticks: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_item_cooldown(&item_id, ticks);
        Ok(())
    }

    fn get_item_cooldown(
        &mut self,
        player: Resource<Player>,
        item_id: String,
    ) -> wasmtime::Result<Option<i32>> {
        let player = self.get(&player)?;
        Ok(player.get_item_cooldown(&item_id))
    }

    fn has_item_cooldown(
        &mut self,
        player: Resource<Player>,
        item_id: String,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        Ok(player.has_item_cooldown(&item_id))
    }

    fn ray_trace_block(
        &mut self,
        player: Resource<Player>,
        max_distance: f64,
        include_fluids: bool,
    ) -> wasmtime::Result<Option<lithummc::plugin::world::RayTraceBlockResult>> {
        let player = self.get(&player)?;
        let start = player.living_entity.entity.get_eye_pos();
        let direction = player.living_entity.entity.get_looking_vector();
        let end = start + direction * max_distance;
        let world = player.living_entity.entity.world.load_full();

        let hit = world.ray_trace_block(start, end, include_fluids);

        Ok(hit.map(
            |(pos, face, hit_pos)| lithummc::plugin::world::RayTraceBlockResult {
                pos: lithummc::plugin::world::BlockPos {
                    x: pos.0.x,
                    y: pos.0.y,
                    z: pos.0.z,
                },
                face: crate::world::to_wasm_block_direction(face),
                hit_pos: to_wasm_position(hit_pos),
            },
        ))
    }

    fn ray_trace_entity(
        &mut self,
        player: Resource<Player>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<lithummc::plugin::world::RayTraceEntityResult>> {
        let player = self.get(&player)?;
        let start = player.living_entity.entity.get_eye_pos();
        let direction = player.living_entity.entity.get_looking_vector();
        let end = start + direction * max_distance;
        let world = player.living_entity.entity.world.load_full();
        let self_id = player.living_entity.entity.entity_id;

        let hits = world.ray_trace_entities(start, end);
        for (hit_entity, hit_pos, distance) in hits {
            if hit_entity.get_entity().entity_id != self_id {
                let entity_res = self
                    .add(hit_entity)
                    .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?;
                return Ok(Some(lithummc::plugin::world::RayTraceEntityResult {
                    entity: entity_res,
                    hit_pos: to_wasm_position(hit_pos),
                    distance,
                }));
            }
        }

        Ok(None)
    }

    fn get_target_entity(
        &mut self,
        player: Resource<Player>,
        max_distance: f64,
    ) -> wasmtime::Result<Option<Resource<lithummc::plugin::world::Entity>>> {
        let res = self.ray_trace_entity(player, max_distance)?;
        Ok(res.map(|r| r.entity))
    }

    fn get_target_block(
        &mut self,
        player: Resource<Player>,
        max_distance: u32,
    ) -> wasmtime::Result<Option<crate::lithummc::plugin::common::Position>> {
        let player = self.get(&player)?;
        let res = player.get_target_block(
            &player.living_entity.entity.world.load_full(),
            f64::from(max_distance),
        );
        Ok(res.map(|p| {
            let vec3 = lithummc_util::math::vector3::Vector3::new(
                f64::from(p.0.x),
                f64::from(p.0.y),
                f64::from(p.0.z),
            );
            to_wasm_position(vec3)
        }))
    }

    fn get_target_block_exact(
        &mut self,
        player: Resource<Player>,
        max_distance: f64,
        include_fluids: bool,
    ) -> wasmtime::Result<Option<lithummc::plugin::world::RayTraceBlockResult>> {
        self.ray_trace_block(player, max_distance, include_fluids)
    }

    fn launch_projectile(
        &mut self,
        _player: Resource<Player>,
        _type_: crate::lithummc::plugin::player::ProjectileType,
    ) -> wasmtime::Result<Option<Resource<crate::lithummc::plugin::world::Entity>>> {
        Ok(None)
    }

    fn set_tab_list_header_footer(
        &mut self,
        player: Resource<Player>,
        header: Resource<lithummc::plugin::text::TextComponent>,
        footer: Resource<lithummc::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let header = self.take(header)?;
        let footer = self.take(footer)?;
        let player = self.get(&player)?;
        player.set_tab_list_header_footer(&header, &footer);
        Ok(())
    }

    fn set_tab_list_order(&mut self, player: Resource<Player>, order: i32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_tab_list_order(order);
        Ok(())
    }

    fn set_tab_list_latency(
        &mut self,
        player: Resource<Player>,
        latency: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_tab_list_latency(latency);
        Ok(())
    }

    fn set_tab_list_listed(
        &mut self,
        player: Resource<Player>,
        listed: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_tab_list_listed(listed);
        Ok(())
    }

    fn show_title(
        &mut self,
        player: Resource<Player>,
        text: Resource<lithummc::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        let player = self.get(&player)?;
        player.show_title(&component, &TitleMode::Title);
        Ok(())
    }

    fn show_subtitle(
        &mut self,
        player: Resource<Player>,
        text: Resource<lithummc::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        let player = self.get(&player)?;
        player.show_title(&component, &TitleMode::SubTitle);
        Ok(())
    }

    fn show_actionbar(
        &mut self,
        player: Resource<Player>,
        text: Resource<lithummc::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        let player = self.get(&player)?;
        player.show_title(&component, &TitleMode::ActionBar);
        Ok(())
    }

    fn send_title_animation(
        &mut self,
        player: Resource<Player>,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.send_title_animation(fade_in, stay, fade_out);
        Ok(())
    }

    async fn transfer(
        &mut self,
        player: Resource<Player>,
        host: String,
        port: u16,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
            client
                .send_packet(&lithummc_protocol::java::client::play::CTransfer::new(
                    &host,
                    lithummc_protocol::codec::var_int::VarInt(i32::from(port)),
                ))
                .await;
        }
        Ok(())
    }

    fn get_selected_slot(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = self.get(&player)?;
        Ok(player.inventory.get_selected_slot())
    }

    fn get_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.living_entity.health.load())
    }

    fn set_health(&mut self, player: Resource<Player>, health: f32) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_health(health);
        Ok(())
    }

    fn get_max_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.living_entity.get_max_health())
    }

    fn set_max_health(
        &mut self,
        player: Resource<Player>,
        max_health: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_max_health(max_health);
        Ok(())
    }

    fn get_food_level(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = self.get(&player)?;
        Ok(player.hunger_manager.level.load())
    }

    fn get_saturation(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.hunger_manager.saturation.load())
    }

    fn set_saturation(
        &mut self,
        player: Resource<Player>,
        saturation: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_saturation(saturation);
        Ok(())
    }

    fn get_exhaustion(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.get_exhaustion())
    }

    fn set_exhaustion(
        &mut self,
        player: Resource<Player>,
        exhaustion: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_exhaustion(exhaustion);
        Ok(())
    }

    fn get_absorption(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.get_absorption())
    }

    fn set_absorption(
        &mut self,
        player: Resource<Player>,
        absorption: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        player.set_absorption(absorption);
        Ok(())
    }

    fn get_experience_level(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = self.get(&player)?;
        Ok(player.experience_level.load(Ordering::Relaxed))
    }

    fn get_experience_progress(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = self.get(&player)?;
        Ok(player.experience_progress.load())
    }

    fn get_experience_points(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = self.get(&player)?;
        Ok(player.experience_points.load(Ordering::Relaxed))
    }

    fn is_flying(&mut self, player: Resource<Player>) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        Ok(player.is_flying())
    }

    fn set_flying(&mut self, player: Resource<Player>, flying: bool) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        {
            let mut abilities = player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            abilities.flying = flying;
        };
        player.send_abilities_update();
        Ok(())
    }

    fn get_abilities(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<lithummc::plugin::player::PlayerAbilities> {
        let player = self.get(&player)?;
        let abilities = player
            .abilities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(lithummc::plugin::player::PlayerAbilities {
            invulnerable: abilities.invulnerable,
            flying: abilities.flying,
            allow_flying: abilities.allow_flying,
            creative: abilities.creative,
            allow_modify_world: abilities.allow_modify_world,
            fly_speed: abilities.fly_speed,
            walk_speed: abilities.walk_speed,
        })
    }

    fn set_abilities(
        &mut self,
        player: Resource<Player>,
        abilities: lithummc::plugin::player::PlayerAbilities,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        {
            let mut a = player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            a.invulnerable = abilities.invulnerable;
            a.flying = abilities.flying;
            a.allow_flying = abilities.allow_flying;
            a.creative = abilities.creative;
            a.allow_modify_world = abilities.allow_modify_world;
            a.fly_speed = abilities.fly_speed;
            a.walk_speed = abilities.walk_speed;
        };
        player.send_abilities_update();
        Ok(())
    }

    fn get_ip(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = self.get(&player)?;
        Ok(player.get_ip())
    }

    fn get_skin(&mut self, player: Resource<Player>) -> wasmtime::Result<Option<PlayerSkin>> {
        let player = self.get(&player)?;
        Ok(player
            .gameprofile
            .properties
            .load()
            .iter()
            .find(|p| p.name.as_ref() == "textures")
            .map(|p| PlayerSkin {
                value: p.value.to_string(),
                signature: p.signature.as_ref().map(std::string::ToString::to_string),
            }))
    }

    fn set_skin(&mut self, player: Resource<Player>, skin: PlayerSkin) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let mut properties = (**player.gameprofile.properties.load()).clone();

        properties.retain(|p| p.name.as_ref() != "textures");
        properties.push(Property {
            name: "textures".into(),
            value: skin.value.into(),
            signature: skin.signature.map(std::convert::Into::into),
        });

        player.gameprofile.properties.store(Arc::new(properties));

        Ok(())
    }

    fn get_skin_parts(&mut self, player: Resource<Player>) -> wasmtime::Result<SkinParts> {
        let player = self.get(&player)?;
        let mask = player.config.load().skin_parts;
        let mut parts = SkinParts::empty();
        if mask & 0x01 != 0 {
            parts |= SkinParts::CAPE;
        }
        if mask & 0x02 != 0 {
            parts |= SkinParts::JACKET;
        }
        if mask & 0x04 != 0 {
            parts |= SkinParts::LEFT_SLEEVE;
        }
        if mask & 0x08 != 0 {
            parts |= SkinParts::RIGHT_SLEEVE;
        }
        if mask & 0x10 != 0 {
            parts |= SkinParts::LEFT_PANTS_LEG;
        }
        if mask & 0x20 != 0 {
            parts |= SkinParts::RIGHT_PANTS_LEG;
        }
        if mask & 0x40 != 0 {
            parts |= SkinParts::HAT;
        }
        Ok(parts)
    }

    fn set_skin_parts(
        &mut self,
        player: Resource<Player>,
        parts: SkinParts,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let mut mask = 0u8;
        if parts.contains(SkinParts::CAPE) {
            mask |= 0x01;
        }
        if parts.contains(SkinParts::JACKET) {
            mask |= 0x02;
        }
        if parts.contains(SkinParts::LEFT_SLEEVE) {
            mask |= 0x04;
        }
        if parts.contains(SkinParts::RIGHT_SLEEVE) {
            mask |= 0x08;
        }
        if parts.contains(SkinParts::LEFT_PANTS_LEG) {
            mask |= 0x10;
        }
        if parts.contains(SkinParts::RIGHT_PANTS_LEG) {
            mask |= 0x20;
        }
        if parts.contains(SkinParts::HAT) {
            mask |= 0x40;
        }

        {
            let mut config = (**player.config.load()).clone();
            config.skin_parts = mask;
            player.config.store(Arc::new(config));
        };
        player.send_client_information();
        Ok(())
    }

    fn get_advancement_progress(
        &mut self,
        player: Resource<Player>,
        advancement_id: String,
    ) -> wasmtime::Result<Option<lithummc::plugin::advancement::AdvancementProgress>> {
        let player = self.get(&player)?;
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(None);
        };
        let guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let progress = guard.progress.map.get(advancement).map_or_else(
            || lithummc::plugin::advancement::AdvancementProgress {
                advancement_id: advancement.id.to_string(),
                done: false,
                awarded_criteria: Vec::new(),
                remaining_criteria: advancement
                    .criteria
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            },
            |progress| lithummc::plugin::advancement::AdvancementProgress {
                advancement_id: advancement.id.to_string(),
                done: progress.is_done(),
                awarded_criteria: progress
                    .get_completed_criteria()
                    .map(|s| s.to_string())
                    .collect(),
                remaining_criteria: progress
                    .get_remaining_criteria()
                    .map(|s| s.to_string())
                    .collect(),
            },
        );
        Ok(Some(progress))
    }

    fn revoke_advancement_criterion(
        &mut self,
        player: Resource<Player>,
        advancement_id: String,
        criterion: String,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(false);
        };
        let mut guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let revoked = guard.revoke(advancement, &criterion);
        if revoked {
            guard.flush_dirty(player, true);
        }
        Ok(revoked)
    }

    fn revoke_advancement(
        &mut self,
        player: Resource<Player>,
        advancement_id: String,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(false);
        };
        let mut guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let progress = guard.progress.get_mut_or_start_progress(advancement);
        if !progress.has_progress() {
            return Ok(false);
        }
        let completed: Vec<Arc<str>> = progress.get_completed_criteria().collect();
        let mut any_revoked = false;
        for criterion in completed {
            if guard.revoke(advancement, &criterion) {
                any_revoked = true;
            }
        }
        if any_revoked {
            guard.flush_dirty(player, true);
        }
        Ok(any_revoked)
    }

    fn has_advancement(
        &mut self,
        player: Resource<Player>,
        advancement_id: String,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?;
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(false);
        };
        let guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let done =
            guard.progress.map.get(advancement).is_some_and(
                lithummc_core::entity::player::advancement::AdvancementProgress::is_done,
            );
        Ok(done)
    }

    fn get_completed_advancements(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Vec<String>> {
        let player = self.get(&player)?;
        let guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let list = guard
            .progress
            .map
            .iter()
            .filter(|(_, p)| p.is_done())
            .map(|(adv, _)| adv.id.to_string())
            .collect();
        Ok(list)
    }

    fn get_selected_advancement_tab(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<String>> {
        let player = self.get(&player)?;
        let guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(guard.last_selected_tab.map(|adv| adv.id.to_string()))
    }

    fn set_selected_advancement_tab(
        &mut self,
        player: Resource<Player>,
        tab_id: Option<String>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?;
        let target_adv = tab_id
            .as_deref()
            .and_then(crate::advancement::find_advancement);
        let mut guard = player
            .advancements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.set_selected_tab(target_adv);
        Ok(())
    }

    fn as_java(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<lithummc::plugin::player::JavaPlayer>>> {
        let player = self.get(&player)?.clone();
        if let lithummc_core::net::ClientPlatform::Java(_) = player.client.as_ref() {
            Ok(Some(self.add(player)?))
        } else {
            Ok(None)
        }
    }

    fn as_bedrock(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<lithummc::plugin::player::BedrockPlayer>>> {
        let player = self.get(&player)?.clone();
        if let lithummc_core::net::ClientPlatform::Bedrock(_) = player.client.as_ref() {
            Ok(Some(self.add(player)?))
        } else {
            Ok(None)
        }
    }

    fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::player::HostPlayerWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn open_ender_chest(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.open_ender_chest();
            })
            .await
    }

    async fn set_gamemode(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        mode: lithummc::plugin::common::GameMode,
    ) -> wasmtime::Result<bool> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let mode = from_wasm_game_mode(mode);

        plugin
            .store
            .pump_blocking(&mut host, move || player.set_gamemode(mode))
            .await
    }

    async fn set_permission_level(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        level: lithummc::plugin::permission::PermissionLevel,
    ) -> wasmtime::Result<()> {
        let (player, server, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.server.as_ref().expect("server not available").clone(),
                plugin_from_state(state)?,
            )
        };
        let level = from_wit_permission_level(level);

        plugin
            .store
            .pump_blocking(&mut host, move || {
                let command_dispatcher = server.command_dispatcher.load();
                player.set_permission_lvl(&server, level, &command_dispatcher);
            })
            .await
    }

    async fn has_permission(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<bool> {
        let (player, server, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.server.as_ref().expect("server not available").clone(),
                plugin_from_state(state)?,
            )
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.has_permission(&server, &node))
            .await
    }

    async fn add_effect(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        effect: lithummc::plugin::status_effect::StatusEffectInstance,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let effect_type = super::status_effect::from_wasm_status_effect_type(effect.effect_type);
        let Some(status_effect) =
            lithummc_data::effect::StatusEffect::from_name(effect_type.to_name())
        else {
            return Ok(());
        };
        let effect = lithummc_data::potion::Effect {
            effect_type: status_effect,
            duration: effect.duration as i32,
            amplifier: effect.amplifier,
            ambient: effect.ambient,
            show_particles: effect.show_particles,
            show_icon: effect.show_icon,
            blend: false,
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.add_effect(effect))
            .await
    }

    async fn heal(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        amount: f32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.heal(amount))
            .await
    }

    async fn damage(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        amount: f32,
        damage_type: WitDamageType,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let damage_type = from_wit_damage_type(damage_type);

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.damage(&*player, amount, damage_type);
            })
            .await
    }

    async fn kill(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.kill())
            .await
    }

    async fn teleport(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        position: lithummc::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Resource<World>,
    ) -> wasmtime::Result<()> {
        let (player, world, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.take(world)?,
                plugin_from_state(state)?,
            )
        };
        let position = from_wasm_position(position);

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.teleport(position, yaw, pitch, world);
            })
            .await
    }

    async fn teleport_world(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        world: Resource<World>,
        position: lithummc::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) -> wasmtime::Result<()> {
        let (player, world, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.take(world)?,
                plugin_from_state(state)?,
            )
        };
        let position = from_wasm_position(position);
        let runtime = tokio::runtime::Handle::current();

        plugin
            .store
            .pump_blocking(&mut host, move || {
                runtime.block_on(player.teleport_world(world, position, yaw, pitch));
            })
            .await
    }

    async fn respawn(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let runtime = tokio::runtime::Handle::current();

        plugin
            .store
            .pump_blocking(&mut host, move || runtime.block_on(player.respawn()))
            .await
    }

    async fn open_gui(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        gui: Resource<lithummc::plugin::gui::Gui>,
    ) -> wasmtime::Result<()> {
        let (player, gui, plugin) = {
            let state = host.get();
            let gui = state.take(gui)?;
            (state.get(&player)?.clone(), gui, plugin_from_state(state)?)
        };

        let (window_type, inventory, allow_grab_items, allow_put_items, title) = plugin
            .store
            .pump_reentry(&mut host, async move {
                let gui = gui.lock().await;
                (
                    gui.window_type,
                    gui.inventory.clone(),
                    gui.allow_grab_items,
                    gui.allow_put_items,
                    gui.title.clone(),
                )
            })
            .await?;

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.increment_screen_handler_sync_id();
                let sync_id = player.screen_handler_sync_id.load(Ordering::Relaxed);
                let screen_handler: Arc<
                    std::sync::Mutex<dyn lithummc_inventory::screen_handler::ScreenHandler>,
                > = Arc::new(std::sync::Mutex::new(PluginScreenHandler::new(
                    sync_id,
                    window_type,
                    player.inventory(),
                    &inventory,
                    allow_grab_items,
                    allow_put_items,
                )));
                player.open_handled_screen_direct(screen_handler, &title);
            })
            .await
    }

    async fn ban(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        options: lithummc::plugin::player::BanPlayerOptions,
    ) -> wasmtime::Result<()> {
        let lithummc::plugin::player::BanPlayerOptions {
            reason,
            source,
            expires_at_utc,
            duration_seconds,
            kick_if_online,
            log_to_console,
        } = options;
        let (player, server, reason, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.server.as_ref().expect("server not available").clone(),
                reason.map(|t| state.take(t)).transpose()?,
                plugin_from_state(state)?,
            )
        };
        let expires = parse_ban_expiry(expires_at_utc, duration_seconds);

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.ban_explicit(
                    &server,
                    reason,
                    source,
                    expires,
                    kick_if_online,
                    log_to_console,
                );
            })
            .await
    }

    async fn ban_ip(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        options: lithummc::plugin::player::BanIpOptions,
    ) -> wasmtime::Result<()> {
        let lithummc::plugin::player::BanIpOptions {
            reason,
            source,
            expires_at_utc,
            duration_seconds,
            kick_matching_players,
            log_to_console,
        } = options;
        let (player, server, reason, plugin) = {
            let state = host.get();
            (
                state.get(&player)?.clone(),
                state.server.as_ref().expect("server not available").clone(),
                reason.map(|t| state.take(t)).transpose()?,
                plugin_from_state(state)?,
            )
        };
        let expires = parse_ban_expiry(expires_at_utc, duration_seconds);

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.ban_ip_explicit(
                    &server,
                    reason,
                    source,
                    expires,
                    kick_matching_players,
                    log_to_console,
                );
            })
            .await
    }

    async fn set_food_level(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        level: u8,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.set_food_level(level))
            .await
    }

    async fn set_experience_level(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        level: i32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.set_experience_level(level, true))
            .await
    }

    async fn set_experience_progress(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        progress: f32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.set_experience(
                    player.experience_level.load(Ordering::Relaxed),
                    progress,
                    player.experience_points.load(Ordering::Relaxed),
                );
            })
            .await
    }

    async fn set_experience_points(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || {
                player.set_experience(
                    player.experience_level.load(Ordering::Relaxed),
                    player.experience_progress.load(),
                    points,
                );
            })
            .await
    }

    async fn add_experience_levels(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        levels: i32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.add_experience_levels(levels))
            .await
    }

    async fn add_experience_points(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || player.add_experience_points(points))
            .await
    }

    async fn award_advancement_criterion(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        advancement_id: String,
        criterion: String,
    ) -> wasmtime::Result<bool> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(false);
        };

        plugin
            .store
            .pump_blocking(&mut host, move || {
                let result = {
                    let mut guard = player
                        .advancements
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.award(advancement, &criterion)
                };

                PlayerAdvancement::finish_award(&player, advancement, result);
                if result.awarded() {
                    let mut guard = player
                        .advancements
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.flush_dirty(&player, true);
                }
                result.awarded()
            })
            .await
    }

    async fn award_advancement(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<Player>,
        advancement_id: String,
    ) -> wasmtime::Result<bool> {
        let (player, plugin) = {
            let state = host.get();
            (state.get(&player)?.clone(), plugin_from_state(state)?)
        };
        let Some(advancement) = crate::advancement::find_advancement(&advancement_id) else {
            return Ok(false);
        };

        plugin
            .store
            .pump_blocking(&mut host, move || {
                let result = {
                    let mut guard = player
                        .advancements
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let progress = guard.progress.get_mut_or_start_progress(advancement);
                    if progress.is_done() {
                        return false;
                    }
                    let remaining: Vec<Arc<str>> = progress.get_remaining_criteria().collect();
                    let mut result = AdvancementAward::default();
                    for criterion in remaining {
                        result = result.combine(guard.award(advancement, &criterion));
                    }
                    result
                };

                PlayerAdvancement::finish_award(&player, advancement, result);
                if result.awarded() {
                    let mut guard = player
                        .advancements
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.flush_dirty(&player, true);
                }
                result.awarded()
            })
            .await
    }
}

impl lithummc::plugin::player::HostJavaPlayer for PluginHostState {
    fn get_version(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<lithummc::plugin::player::JavaMinecraftVersion> {
        let player = self.get(&player)?.clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(to_wasm_java_version(client.version.load()))
    }

    fn get_brand(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<String> {
        let player = self.get(&player)?.clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(client.brand.load().as_ref().clone().unwrap_or_default())
    }

    fn get_server_address(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<String> {
        let player = self.get(&player)?.clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(client.server_address.clone())
    }

    fn get_settings(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<lithummc::plugin::player::JavaPlayerSettings> {
        let player = self.get(&player)?.clone();

        let config = player.config.load();
        let mask = config.skin_parts;
        let mut parts = SkinParts::empty();
        if mask & 0x01 != 0 {
            parts |= SkinParts::CAPE;
        }
        if mask & 0x02 != 0 {
            parts |= SkinParts::JACKET;
        }
        if mask & 0x04 != 0 {
            parts |= SkinParts::LEFT_SLEEVE;
        }
        if mask & 0x08 != 0 {
            parts |= SkinParts::RIGHT_SLEEVE;
        }
        if mask & 0x10 != 0 {
            parts |= SkinParts::LEFT_PANTS_LEG;
        }
        if mask & 0x20 != 0 {
            parts |= SkinParts::RIGHT_PANTS_LEG;
        }
        if mask & 0x40 != 0 {
            parts |= SkinParts::HAT;
        }

        Ok(lithummc::plugin::player::JavaPlayerSettings {
            locale: config.locale.clone(),
            view_distance: config.view_distance.get(),
            chat_mode: to_wasm_chat_mode(&config.chat_mode),
            chat_colors: config.chat_colors,
            skin_parts: parts,
            main_hand: match config.main_hand {
                lithummc_util::Hand::Left => lithummc::plugin::common::Hand::Left,
                lithummc_util::Hand::Right => lithummc::plugin::common::Hand::Right,
            },
            text_filtering: config.text_filtering,
            server_listing: config.server_listing,
        })
    }

    async fn send_packet(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        packet: lithummc::plugin::java_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        if let Some(bytes) = crate::generated_packets::serialize_java_packet(
            &packet,
            lithummc_data::packet::CURRENT_MC_VERSION,
        ) {
            client.send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn send_custom_payload(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        channel: String,
        data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        if let lithummc_core::net::ClientPlatform::Java(_) = player.client.as_ref() {
            player
                .send_client_packet(&lithummc_protocol::java::client::play::CCustomPayload::new(
                    &channel, &data,
                ))
                .await;
        }
        Ok(())
    }

    fn get_scoreboard(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<Resource<lithummc::plugin::scoreboard::Scoreboard>> {
        let player = self.get(&player)?.clone();
        self.add(lithummc_wasm_host_common::state::ScoreboardProvider::Player(
            player,
        ))
    }

    fn reset_scoreboard(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();
        player.reset_scoreboard();
        Ok(())
    }

    async fn send_resource_pack(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        pack: lithummc::plugin::player::JavaResourcePack,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let uuid = Uuid::from_wit(&pack.id);
        let prompt_message = pack.prompt_message.map(|t| self.take(t)).transpose()?;

        if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
            client
                .send_packet(
                    &lithummc_protocol::java::client::play::CAddResourcePack::new(
                        &uuid,
                        &pack.url,
                        &pack.hash,
                        pack.forced,
                        prompt_message,
                    ),
                )
                .await;
        }
        Ok(())
    }

    async fn remove_resource_pack(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        id: lithummc::plugin::uuid::Uuid,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let uuid = Uuid::from_wit(&id);

        if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
            client
                .send_packet(
                    &lithummc_protocol::java::client::play::CRemoveResourcePack::new(Some(&uuid)),
                )
                .await;
        }
        Ok(())
    }

    async fn clear_resource_packs(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
            client
                .send_packet(&lithummc_protocol::java::client::play::CRemoveResourcePack::new(None))
                .await;
        }
        Ok(())
    }

    fn send_game_event(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        event: lithummc::plugin::player::ClientGameEvent,
        value: f32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let internal_event = from_wit_client_game_event(event);
        player.send_game_event(internal_event, value);
        Ok(())
    }

    fn send_entity_status(
        &mut self,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        entity_id: i32,
        status: lithummc::plugin::entity_statuses::EntityStatus,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        // SAFETY: The WIT enum variants and discriminants are 1:1 generated from entity_statuses.json
        let internal_status: lithummc_data::entity_status::EntityStatus =
            unsafe { std::mem::transmute(status as u8) };
        let packet = lithummc_protocol::java::client::play::CEntityStatus::new(
            entity_id,
            internal_status as u8 as i8,
        );
        player.try_send_client_packet(&packet);
        Ok(())
    }

    fn drop(&mut self, rep: Resource<lithummc::plugin::player::JavaPlayer>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::player::HostJavaPlayerWithStore<PluginHostState>
    for HasSelf<PluginHostState>
{
    #[allow(clippy::too_many_lines)]
    async fn show_dialog(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        dialog: Dialog,
    ) -> wasmtime::Result<()> {
        let (player, protocol_dialog, plugin) = {
            let state = host.get();
            let player = state.get(&player)?.clone();
            let protocol_dialog = super::events::dialog::protocol_dialog_from_wasm(state, &dialog);
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (player, protocol_dialog, plugin)
        };

        let operation = async move {
            if let Some(server) = player.world().server.upgrade() {
                let mut event =
                    lithummc_core::plugin::api::events::dialog::dialog_show::DialogShowEvent::new(
                        player.clone(),
                        protocol_dialog.clone(),
                    );
                server.plugin_manager.fire(&server, &mut event).await;
                if event.cancelled {
                    return Ok(());
                }
            }

            if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
                match client.connection_state.load() {
                    lithummc_protocol::ConnectionState::Config => {
                        client
                            .send_packet(
                                &lithummc_protocol::java::client::config::CConfigShowDialog::new(
                                    lithummc_protocol::IdOr::Value(DialogNBT::from_dialog(
                                        &protocol_dialog,
                                    )),
                                ),
                            )
                            .await;
                    }
                    lithummc_protocol::ConnectionState::Play => {
                        client
                            .send_packet(
                                &lithummc_protocol::java::client::play::CPlayShowDialog::new(
                                    lithummc_protocol::IdOr::Value(DialogNBT::from_dialog(
                                        &protocol_dialog,
                                    )),
                                ),
                            )
                            .await;
                    }
                    _ => {}
                }
            }

            Ok(())
        };

        plugin.store.pump_reentry(&mut host, operation).await?
    }

    async fn clear_dialog(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            let player = state.get(&player)?.clone();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (player, plugin)
        };

        let operation = async move {
            if let Some(server) = player.world().server.upgrade() {
                let mut event =
                    lithummc_core::plugin::api::events::dialog::dialog_clear::DialogClearEvent::new(
                        player.clone(),
                    );
                server.plugin_manager.fire(&server, &mut event).await;
                if event.cancelled {
                    return Ok(());
                }
            }

            if let lithummc_core::net::ClientPlatform::Java(client) = player.client.as_ref() {
                match client.connection_state.load() {
                    lithummc_protocol::ConnectionState::Config => {
                        client
                            .send_packet(
                                &lithummc_protocol::java::client::config::CConfigClearDialog::new(),
                            )
                            .await;
                    }
                    lithummc_protocol::ConnectionState::Play => {
                        client
                            .send_packet(
                                &lithummc_protocol::java::client::play::CPlayClearDialog::new(),
                            )
                            .await;
                    }
                    _ => {}
                }
            }

            Ok(())
        };

        plugin.store.pump_reentry(&mut host, operation).await?
    }

    async fn kick(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<lithummc::plugin::player::JavaPlayer>,
        options: lithummc::plugin::player::JavaKickOptions,
    ) -> wasmtime::Result<()> {
        let (player, reason, plugin) = {
            let state = host.get();
            let player = state.get(&player)?.clone();
            let reason = state.take(options.reason)?;
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (player, reason, plugin)
        };

        if options.log_to_console {
            tracing::info!(
                "Kicking Java player {} ({}): {}",
                player.gameprofile.name,
                player.gameprofile.id,
                reason.clone().to_pretty_console()
            );
        }

        let operation = async move {
            if let Some(server) = player.world().server.upgrade()
                && let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id)
            {
                let mut event =
                    lithummc_core::plugin::api::events::player::player_kick::PlayerKickEvent::new(
                        player_arc,
                        reason.clone().to_pretty_console(),
                    );
                server.plugin_manager.fire(&server, &mut event).await;
                if event.cancelled {
                    return Ok(());
                }
            }

            let send_packet = options.teardown_policy
                != lithummc::plugin::player::SocketTeardownPolicy::DropConnection;
            if let lithummc_core::net::ClientPlatform::Java(java) = player.client.as_ref() {
                java.kick_explicit(&reason, send_packet).await;
            }

            Ok(())
        };

        plugin.store.pump_reentry(&mut host, operation).await?
    }
}

impl lithummc::plugin::player::HostBedrockPlayer for PluginHostState {
    fn get_version(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<lithummc::plugin::player::BedrockMinecraftVersion> {
        let player = self.get(&player)?;

        if let lithummc_core::net::ClientPlatform::Bedrock(client) = player.client.as_ref() {
            Ok(to_wasm_bedrock_version(client.version.load()))
        } else {
            Ok(lithummc::plugin::player::BedrockMinecraftVersion::Unknown)
        }
    }

    fn get_ability(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        ability: lithummc::plugin::player::BedrockAbility,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?.clone();

        let ability = from_wasm_bedrock_ability(ability);
        let abilities = player
            .abilities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        match ability {
            lithummc_protocol::bedrock::client::update_abilities::Ability::Build
            | lithummc_protocol::bedrock::client::update_abilities::Ability::Mine => {
                Ok(abilities.allow_modify_world)
            }
            lithummc_protocol::bedrock::client::update_abilities::Ability::Invulnerable => {
                Ok(abilities.invulnerable)
            }
            lithummc_protocol::bedrock::client::update_abilities::Ability::Flying => {
                Ok(abilities.flying)
            }
            lithummc_protocol::bedrock::client::update_abilities::Ability::MayFly => {
                Ok(abilities.allow_flying)
            }
            lithummc_protocol::bedrock::client::update_abilities::Ability::Instabuild => {
                Ok(abilities.creative)
            }
            _ => Ok(false), // Most Bedrock-specific abilities aren't tracked in generic Abilities struct yet
        }
    }

    fn set_ability(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        ability: lithummc::plugin::player::BedrockAbility,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let ability = from_wasm_bedrock_ability(ability);
        {
            let mut abilities = player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match ability {
                lithummc_protocol::bedrock::client::update_abilities::Ability::Build
                | lithummc_protocol::bedrock::client::update_abilities::Ability::Mine => {
                    abilities.allow_modify_world = value;
                }
                lithummc_protocol::bedrock::client::update_abilities::Ability::Invulnerable => {
                    abilities.invulnerable = value;
                }
                lithummc_protocol::bedrock::client::update_abilities::Ability::Flying => {
                    abilities.flying = value;
                }
                lithummc_protocol::bedrock::client::update_abilities::Ability::MayFly => {
                    abilities.allow_flying = value;
                }
                lithummc_protocol::bedrock::client::update_abilities::Ability::Instabuild => {
                    abilities.creative = value;
                }
                _ => {} // Not supported yet
            }
        };
        player.send_abilities_update();
        Ok(())
    }

    fn get_status_flag(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        flag: lithummc::plugin::player::BedrockStatusFlag,
    ) -> wasmtime::Result<bool> {
        let player = self.get(&player)?.clone();

        let flag_index = from_wasm_bedrock_status_flag(flag);
        if flag_index < 64 {
            let flags = player
                .living_entity
                .entity
                .bedrock_flags
                .load(Ordering::Relaxed);
            Ok((flags & (1 << flag_index)) != 0)
        } else {
            let flags = player
                .living_entity
                .entity
                .bedrock_flags_two
                .load(Ordering::Relaxed);
            Ok((flags & (1 << (flag_index - 64))) != 0)
        }
    }

    async fn set_status_flag(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        flag: lithummc::plugin::player::BedrockStatusFlag,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        let flag_index = from_wasm_bedrock_status_flag(flag);
        if flag_index < 64 {
            let mut flags = player
                .living_entity
                .entity
                .bedrock_flags
                .load(Ordering::Relaxed);
            if value {
                flags |= 1 << flag_index;
            } else {
                flags &= !(1 << flag_index);
            }
            player
                .living_entity
                .entity
                .bedrock_flags
                .store(flags, Ordering::Relaxed);
        } else {
            let mut flags = player
                .living_entity
                .entity
                .bedrock_flags_two
                .load(Ordering::Relaxed);
            if value {
                flags |= 1 << (flag_index - 64);
            } else {
                flags &= !(1 << (flag_index - 64));
            }
            player
                .living_entity
                .entity
                .bedrock_flags_two
                .store(flags, Ordering::Relaxed);
        }

        let mut metadata = SyncedActorDataList(std::collections::HashMap::new());
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Int64(
                player
                    .living_entity
                    .entity
                    .bedrock_flags
                    .load(Ordering::Relaxed),
            ),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Int64(
                player
                    .living_entity
                    .entity
                    .bedrock_flags_two
                    .load(Ordering::Relaxed),
            ),
        );

        let packet = CSetActorData {
            target_runtime_id: VarULong(player.get_entity().entity_id as u64),
            actor_data: metadata,
            synced_properties: PropertySyncData {
                int_entries_list: std::collections::HashMap::new(),
                float_entries_list: std::collections::HashMap::new(),
            },
            tick: VarULong(0),
        };

        if let lithummc_core::net::ClientPlatform::Bedrock(client) = player.client.as_ref() {
            client.send_packet(&packet).await;
        }

        Ok(())
    }

    fn get_settings(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<lithummc::plugin::player::BedrockPlayerSettings> {
        let player = self.get(&player)?.clone();

        if let lithummc_core::net::ClientPlatform::Bedrock(client) = player.client.as_ref() {
            let data = client.client_data.load();
            (**data).as_ref().map_or_else(
                || Err(wasmtime::Error::msg("client data not available")),
                |data| {
                    Ok(lithummc::plugin::player::BedrockPlayerSettings {
                        game_version: data.game_version.clone(),
                        device_os: to_wasm_bedrock_device_os(data.device_os),
                        device_id: data.device_id.clone(),
                        device_model: data.device_model.clone(),
                        language_code: data.language_code.clone(),
                        current_input_mode: to_wasm_bedrock_input_mode(data.current_input_mode),
                        default_input_mode: to_wasm_bedrock_input_mode(data.default_input_mode),
                        ui_profile: to_wasm_bedrock_ui_profile(data.ui_profile),
                        gui_scale: data.gui_scale,
                        is_editor_mode: data.client_is_editor_capable,
                        max_view_distance: data.max_view_distance,
                        memory_tier: data.memory_tier,
                        graphics_mode: to_wasm_bedrock_graphics_mode(data.graphics_mode),
                        playfab_id: data.play_fab_id.clone(),
                        client_random_id: data.client_random_id,
                        platform_offline_id: data.platform_offline_id.clone(),
                        platform_online_id: data.platform_online_id.clone(),
                        skin_id: data.skin_id.clone(),
                        arm_size: data.arm_size.clone(),
                        is_persona_skin: data.persona_skin,
                        is_premium_skin: data.premium_skin,
                        is_trusted_skin: data.trusted_skin,
                    })
                },
            )
        } else {
            Err(wasmtime::Error::msg("not a bedrock player"))
        }
    }

    async fn send_packet(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        packet: lithummc::plugin::bedrock_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        if let Some(bytes) = crate::generated_packets::serialize_bedrock_packet(&packet) {
            player.client.send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn open_form(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        form: Form,
    ) -> wasmtime::Result<u32> {
        let player = self.get(&player)?.clone();

        if let lithummc_core::net::ClientPlatform::Bedrock(client) = player.client.as_ref() {
            let form_id = client.next_form_id.fetch_add(1, Ordering::Relaxed);

            let locale_str = player.config.load().locale.clone();
            let locale = Locale::from_str(&locale_str).unwrap_or(Locale::EnUs);

            let form_json = match form {
                Form::Simple(simple) => self.serialize_simple_form(simple, locale),
                Form::Modal(modal) => self.serialize_modal_form(modal, locale),
                Form::Custom(custom) => self.serialize_custom_form(custom, locale),
            };

            client
                .send_packet(&CModalFormRequest {
                    form_id: lithummc_protocol::codec::var_uint::VarUInt(form_id),
                    form_ui_json: form_json.to_string(),
                })
                .await;

            Ok(form_id)
        } else {
            Ok(0)
        }
    }

    fn get_scoreboard(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<Resource<lithummc::plugin::scoreboard::BedrockScoreboard>> {
        let player = self.get(&player)?.clone();
        self.add(player)
    }

    fn reset_scoreboard(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();
        player.reset_scoreboard();
        Ok(())
    }

    async fn send_resource_packs_info(
        &mut self,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        info: lithummc::plugin::player::BedrockResourcePacksInfo,
    ) -> wasmtime::Result<()> {
        let player = self.get(&player)?.clone();

        if let lithummc_core::net::ClientPlatform::Bedrock(client) = player.client.as_ref() {
            let entries = info
                .packs
                .into_iter()
                .map(
                    |p| lithummc_protocol::bedrock::client::resource_packs_info::PackInfoData {
                        pack_id_version: PackIdVersion::new(Uuid::from_wit(&p.id), p.version),
                        pack_size: p.size,
                        cdn_url: p.download_url,
                        content_key: p.content_key.unwrap_or_default(),
                        subpack_name: p.sub_pack_name.unwrap_or_default(),
                        content_identity: p.content_id.unwrap_or_default(),
                        has_scripts: p.has_scripts,
                        is_addon_pack: p.addon_pack,
                        is_ray_tracing_capable: p.rtx_enabled,
                    },
                )
                .collect();

            let world_template_id = info
                .world_template_id
                .map_or_else(uuid::Uuid::nil, |id| Uuid::from_wit(&id));

            let packs_info =
                lithummc_protocol::bedrock::client::resource_packs_info::CResourcePacksInfo {
                    resource_pack_required: info.required,
                    has_addon_packs: info.has_addon_packs,
                    has_scripts: info.has_scripts,
                    force_disable_vibrant_visuals: info.is_vibrant_visuals_force_disabled,
                    world_template_id_and_version: PackIdVersion::new(
                        world_template_id,
                        info.world_template_version.unwrap_or_default(),
                    ),
                    resource_packs: entries,
                };

            client.send_packet(&packs_info).await;
        }
        Ok(())
    }

    fn drop(
        &mut self,
        rep: Resource<lithummc::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::player::HostBedrockPlayerWithStore<PluginHostState>
    for HasSelf<PluginHostState>
{
    async fn kick(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<lithummc::plugin::player::BedrockPlayer>,
        options: lithummc::plugin::player::BedrockKickOptions,
    ) -> wasmtime::Result<()> {
        let (player, plugin) = {
            let state = host.get();
            let player = state.get(&player)?.clone();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (player, plugin)
        };

        let disconnect_reason = from_wasm_bedrock_disconnect_reason(options.reason);
        if options.log_to_console {
            tracing::info!(
                "Kicking Bedrock player {} ({}) [Reason: {:?}]: {}",
                player.gameprofile.name,
                player.gameprofile.id,
                disconnect_reason,
                options.message
            );
        }

        let operation = async move {
            if let Some(server) = player.world().server.upgrade()
                && let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id)
            {
                let mut event =
                    lithummc_core::plugin::api::events::player::player_kick::PlayerKickEvent::new(
                        player_arc,
                        options.message.clone(),
                    );
                server.plugin_manager.fire(&server, &mut event).await;
                if event.cancelled {
                    return Ok(());
                }
            }

            let send_packet = options.teardown_policy
                != lithummc::plugin::player::SocketTeardownPolicy::DropConnection;
            if let lithummc_core::net::ClientPlatform::Bedrock(bedrock) = player.client.as_ref() {
                bedrock
                    .kick_explicit(
                        disconnect_reason,
                        options.message,
                        options.skip_message,
                        options.filtered_message,
                        send_packet,
                    )
                    .await;
            }

            Ok(())
        };

        plugin.store.pump_reentry(&mut host, operation).await?
    }
}
