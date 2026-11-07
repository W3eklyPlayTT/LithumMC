use wasmtime::component::Resource;

use crate::lithummc::{
    self,
    plugin::scoreboard::{
        self, CollisionRule, DisplaySlot, HostBedrockScoreboard, NametagVisibility, RenderType,
        TeamSettings,
    },
};
use lithummc_core::world::scoreboard::{ScoreboardObjective, ScoreboardScore, Team};
use lithummc_protocol::NumberFormat;
use lithummc_protocol::codec::var_int::VarInt;
use lithummc_wasm_host_common::state::{PluginHostState, ScoreboardProvider};

fn map_number_format(
    nf: Option<scoreboard::NumberFormat>,
    state: &PluginHostState,
) -> wasmtime::Result<Option<NumberFormat>> {
    match nf {
        None => Ok(None),
        Some(scoreboard::NumberFormat::Blank) => Ok(Some(NumberFormat::Blank)),
        Some(scoreboard::NumberFormat::Fixed(tc)) => {
            let text = state.get(&tc)?.clone();
            Ok(Some(NumberFormat::Fixed(text)))
        }
    }
}

impl scoreboard::Host for PluginHostState {}

impl scoreboard::HostScoreboard for PluginHostState {
    fn add_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        display_name: Resource<lithummc::plugin::text::TextComponent>,
        render_type: RenderType,
        number_format: Option<scoreboard::NumberFormat>,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let display_name = self.take(display_name)?;
        let nf = map_number_format(number_format, self)?;

        let rt = match render_type {
            RenderType::Integer => lithummc_protocol::java::client::play::RenderType::Integer,
            RenderType::Hearts => lithummc_protocol::java::client::play::RenderType::Hearts,
        };

        let objective = ScoreboardObjective::new(name, display_name, rt, nf, "dummy");

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .add_objective(world.as_ref(), objective);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_objective(player.as_ref(), objective);
                }
            }
        }
        Ok(())
    }

    fn update_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        display_name: Resource<lithummc::plugin::text::TextComponent>,
        render_type: RenderType,
        number_format: Option<scoreboard::NumberFormat>,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let display_name = self.take(display_name)?;
        let nf = map_number_format(number_format, self)?;

        let rt = match render_type {
            RenderType::Integer => lithummc_protocol::java::client::play::RenderType::Integer,
            RenderType::Hearts => lithummc_protocol::java::client::play::RenderType::Hearts,
        };

        let objective = ScoreboardObjective::new(name, display_name, rt, nf, "dummy");

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .update_objective(world.as_ref(), objective);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_objective(player.as_ref(), objective);
                }
            }
        }
        Ok(())
    }

    fn remove_objective(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove_objective(world.as_ref(), &name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_objective(player.as_ref(), &name);
                }
            }
        }
        Ok(())
    }

    fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let slot = map_display_slot(slot);

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .set_display_objective(world.as_ref(), slot, Some(&objective_name));
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.set_display_objective(player.as_ref(), slot, Some(&objective_name));
                }
            }
        }
        Ok(())
    }

    fn clear_display_slot(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        slot: DisplaySlot,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let slot = map_display_slot(slot);

        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clear_display_objective(world.as_ref(), slot);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.clear_display_objective(player.as_ref(), slot);
                }
            }
        }
        Ok(())
    }

    fn update_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
        value: i32,
        number_format: Option<scoreboard::NumberFormat>,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let nf = map_number_format(number_format, self)?;
        let score = ScoreboardScore::new(entity_name, objective_name, VarInt(value), None, nf);
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .update_score(world.as_ref(), score);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_score(player.as_ref(), score);
                }
            }
        }
        Ok(())
    }

    fn add_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
        delta: i32,
    ) -> wasmtime::Result<i32> {
        let provider = self.get(&res)?.clone();
        let new_val = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .add_score(world.as_ref(), entity_name, objective_name, delta),
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                else {
                    return Err(wasmtime::Error::msg("Invalid scoreboard state"));
                };
                sb.add_score(player.as_ref(), entity_name, objective_name, delta)
            }
        };
        Ok(new_val)
    }

    fn remove_score(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove_score(world.as_ref(), &entity_name, &objective_name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_score(player.as_ref(), &entity_name, &objective_name);
                }
            }
        }
        Ok(())
    }

    fn reset_entity_scores(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        entity_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .reset_scores_for_entity(world.as_ref(), &entity_name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.reset_scores_for_entity(player.as_ref(), &entity_name);
                }
            }
        }
        Ok(())
    }

    fn create_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let team = map_team_settings(name, settings, self)?;
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .add_team(world.as_ref(), team);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if !matches!(
                    *custom_guard,
                    Some(lithummc_core::entity::player::CustomScoreboard::Java(_))
                ) {
                    *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Java(
                        lithummc_core::world::scoreboard::Scoreboard::default(),
                    ));
                }
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_team(player.as_ref(), team);
                }
            }
        }
        Ok(())
    }

    fn remove_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove_team(world.as_ref(), &name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_team(player.as_ref(), &name);
                }
            }
        }
        Ok(())
    }

    fn update_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
        settings: TeamSettings,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        let team = map_team_settings(name, settings, self)?;
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .update_team(world.as_ref(), team);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.update_team(player.as_ref(), team);
                }
            }
        }
        Ok(())
    }

    fn add_player_to_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .add_player_to_team(world.as_ref(), &team_name, player_name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.add_player_to_team(player.as_ref(), &team_name, player_name);
                }
            }
        }
        Ok(())
    }

    fn remove_player_from_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
        player_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove_player_from_team(world.as_ref(), &team_name, &player_name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.remove_player_from_team(player.as_ref(), &team_name, &player_name);
                }
            }
        }
        Ok(())
    }

    fn clear_team_players(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
    ) -> wasmtime::Result<()> {
        let provider = self.get(&res)?.clone();
        match provider {
            ScoreboardProvider::World(world) => {
                world
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clear_team_players(world.as_ref(), &team_name);
            }
            ScoreboardProvider::Player(player) => {
                let mut custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_mut()
                {
                    sb.clear_team_players(player.as_ref(), &team_name);
                }
            }
        }
        Ok(())
    }

    fn get_teams(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
    ) -> wasmtime::Result<Vec<String>> {
        let provider = self.get(&res)?.clone();
        let teams = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_teams()
                .keys()
                .cloned()
                .collect(),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_teams().keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
        };
        Ok(teams)
    }

    fn get_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        name: String,
    ) -> wasmtime::Result<Option<TeamSettings>> {
        let provider = self.get(&res)?.clone();
        let team_opt = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_team(&name)
                .cloned(),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_team(&name).cloned()
                } else {
                    None
                }
            }
        };

        if let Some(team) = team_opt {
            Ok(Some(map_team_to_settings(&team, self)?))
        } else {
            Ok(None)
        }
    }

    fn get_team_players(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        team_name: String,
    ) -> wasmtime::Result<Vec<String>> {
        let provider = self.get(&res)?.clone();
        let players = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_team(&team_name)
                .map(|t| t.players.clone())
                .unwrap_or_default(),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_team(&team_name)
                        .map(|t| t.players.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
        };
        Ok(players)
    }

    fn get_player_team(
        &mut self,
        res: Resource<scoreboard::Scoreboard>,
        player_name: String,
    ) -> wasmtime::Result<Option<String>> {
        let provider = self.get(&res)?.clone();
        let team_name = match provider {
            ScoreboardProvider::World(world) => world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_entity_team(&player_name)
                .map(|t| t.name.clone()),
            ScoreboardProvider::Player(player) => {
                let custom_guard = player
                    .custom_scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(lithummc_core::entity::player::CustomScoreboard::Java(sb)) =
                    custom_guard.as_ref()
                {
                    sb.get_entity_team(&player_name).map(|t| t.name.clone())
                } else {
                    None
                }
            }
        };
        Ok(team_name)
    }

    fn drop(&mut self, rep: Resource<scoreboard::Scoreboard>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

const fn map_display_slot(slot: DisplaySlot) -> lithummc_data::scoreboard::ScoreboardDisplaySlot {
    match slot {
        DisplaySlot::PlayerList => lithummc_data::scoreboard::ScoreboardDisplaySlot::List,
        DisplaySlot::Sidebar => lithummc_data::scoreboard::ScoreboardDisplaySlot::Sidebar,
        DisplaySlot::BelowName => lithummc_data::scoreboard::ScoreboardDisplaySlot::BelowName,
        DisplaySlot::SidebarTeamBlack => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamBlack,
        DisplaySlot::SidebarTeamDarkBlue => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkBlue
        }
        DisplaySlot::SidebarTeamDarkGreen => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkGreen
        }
        DisplaySlot::SidebarTeamDarkAqua => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkAqua
        }
        DisplaySlot::SidebarTeamDarkRed => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkRed
        }
        DisplaySlot::SidebarTeamDarkPurple => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkPurple
        }
        DisplaySlot::SidebarTeamGold => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamGold,
        DisplaySlot::SidebarTeamGray => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamGray,
        DisplaySlot::SidebarTeamDarkGray => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamDarkGray
        }
        DisplaySlot::SidebarTeamBlue => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamBlue,
        DisplaySlot::SidebarTeamGreen => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamGreen,
        DisplaySlot::SidebarTeamAqua => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamAqua,
        DisplaySlot::SidebarTeamRed => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamRed,
        DisplaySlot::SidebarTeamLightPurple => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamLightPurple
        }
        DisplaySlot::SidebarTeamYellow => {
            lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamYellow
        }
        DisplaySlot::SidebarTeamWhite => lithummc_data::scoreboard::ScoreboardDisplaySlot::TeamWhite,
    }
}

fn map_team_settings(
    name: String,
    settings: TeamSettings,
    state: &mut PluginHostState,
) -> wasmtime::Result<Team> {
    let display_name = state.take(settings.display_name)?;
    let player_prefix = state.take(settings.prefix)?;
    let player_suffix = state.take(settings.suffix)?;

    let mut options = 0;
    if settings.friendly_fire {
        options |= 0x01;
    }
    if settings.see_friendly_invisibles {
        options |= 0x02;
    }

    Ok(Team {
        name,
        display_name,
        options,
        nametag_visibility: match settings.nametag_visibility {
            NametagVisibility::Always => lithummc_core::world::scoreboard::NameTagVisibility::Always,
            NametagVisibility::Never => lithummc_core::world::scoreboard::NameTagVisibility::Never,
            NametagVisibility::HideForOtherTeams => {
                lithummc_core::world::scoreboard::NameTagVisibility::HideForOtherTeams
            }
            NametagVisibility::HideForOwnTeam => {
                lithummc_core::world::scoreboard::NameTagVisibility::HideForOwnTeam
            }
        },
        collision_rule: match settings.collision_rule {
            CollisionRule::Always => lithummc_core::world::scoreboard::CollisionRule::Always,
            CollisionRule::Never => lithummc_core::world::scoreboard::CollisionRule::Never,
            CollisionRule::PushOtherTeams => {
                lithummc_core::world::scoreboard::CollisionRule::PushOtherTeams
            }
            CollisionRule::PushOwnTeam => {
                lithummc_core::world::scoreboard::CollisionRule::PushOwnTeam
            }
        },
        color: map_named_color(settings.color),
        player_prefix,
        player_suffix,
        players: Vec::new(),
    })
}

const fn map_named_color(
    color: lithummc::plugin::common::NamedColor,
) -> lithummc_util::text::color::NamedColor {
    match color {
        lithummc::plugin::common::NamedColor::Black => lithummc_util::text::color::NamedColor::Black,
        lithummc::plugin::common::NamedColor::DarkBlue => {
            lithummc_util::text::color::NamedColor::DarkBlue
        }
        lithummc::plugin::common::NamedColor::DarkGreen => {
            lithummc_util::text::color::NamedColor::DarkGreen
        }
        lithummc::plugin::common::NamedColor::DarkAqua => {
            lithummc_util::text::color::NamedColor::DarkAqua
        }
        lithummc::plugin::common::NamedColor::DarkRed => {
            lithummc_util::text::color::NamedColor::DarkRed
        }
        lithummc::plugin::common::NamedColor::DarkPurple => {
            lithummc_util::text::color::NamedColor::DarkPurple
        }
        lithummc::plugin::common::NamedColor::Gold => lithummc_util::text::color::NamedColor::Gold,
        lithummc::plugin::common::NamedColor::Gray => lithummc_util::text::color::NamedColor::Gray,
        lithummc::plugin::common::NamedColor::DarkGray => {
            lithummc_util::text::color::NamedColor::DarkGray
        }
        lithummc::plugin::common::NamedColor::Blue => lithummc_util::text::color::NamedColor::Blue,
        lithummc::plugin::common::NamedColor::Green => lithummc_util::text::color::NamedColor::Green,
        lithummc::plugin::common::NamedColor::Aqua => lithummc_util::text::color::NamedColor::Aqua,
        lithummc::plugin::common::NamedColor::Red => lithummc_util::text::color::NamedColor::Red,
        lithummc::plugin::common::NamedColor::LightPurple => {
            lithummc_util::text::color::NamedColor::LightPurple
        }
        lithummc::plugin::common::NamedColor::Yellow => {
            lithummc_util::text::color::NamedColor::Yellow
        }
        lithummc::plugin::common::NamedColor::White => lithummc_util::text::color::NamedColor::White,
    }
}

fn map_team_to_settings(
    team: &Team,
    state: &mut PluginHostState,
) -> wasmtime::Result<TeamSettings> {
    let display_name = state.add(team.display_name.clone())?;
    let prefix = state.add(team.player_prefix.clone())?;
    let suffix = state.add(team.player_suffix.clone())?;

    let friendly_fire = (team.options & 0x01) != 0;
    let see_friendly_invisibles = (team.options & 0x02) != 0;

    let nametag_visibility = match team.nametag_visibility {
        lithummc_core::world::scoreboard::NameTagVisibility::Always => NametagVisibility::Always,
        lithummc_core::world::scoreboard::NameTagVisibility::Never => NametagVisibility::Never,
        lithummc_core::world::scoreboard::NameTagVisibility::HideForOtherTeams => {
            NametagVisibility::HideForOtherTeams
        }
        lithummc_core::world::scoreboard::NameTagVisibility::HideForOwnTeam => {
            NametagVisibility::HideForOwnTeam
        }
    };

    let collision_rule = match team.collision_rule {
        lithummc_core::world::scoreboard::CollisionRule::Always => CollisionRule::Always,
        lithummc_core::world::scoreboard::CollisionRule::Never => CollisionRule::Never,
        lithummc_core::world::scoreboard::CollisionRule::PushOtherTeams => {
            CollisionRule::PushOtherTeams
        }
        lithummc_core::world::scoreboard::CollisionRule::PushOwnTeam => CollisionRule::PushOwnTeam,
    };

    let color = map_named_color_rev(team.color);

    Ok(TeamSettings {
        display_name,
        friendly_fire,
        see_friendly_invisibles,
        nametag_visibility,
        collision_rule,
        color,
        prefix,
        suffix,
    })
}

const fn map_named_color_rev(
    color: lithummc_util::text::color::NamedColor,
) -> lithummc::plugin::common::NamedColor {
    match color {
        lithummc_util::text::color::NamedColor::Black => lithummc::plugin::common::NamedColor::Black,
        lithummc_util::text::color::NamedColor::DarkBlue => {
            lithummc::plugin::common::NamedColor::DarkBlue
        }
        lithummc_util::text::color::NamedColor::DarkGreen => {
            lithummc::plugin::common::NamedColor::DarkGreen
        }
        lithummc_util::text::color::NamedColor::DarkAqua => {
            lithummc::plugin::common::NamedColor::DarkAqua
        }
        lithummc_util::text::color::NamedColor::DarkRed => {
            lithummc::plugin::common::NamedColor::DarkRed
        }
        lithummc_util::text::color::NamedColor::DarkPurple => {
            lithummc::plugin::common::NamedColor::DarkPurple
        }
        lithummc_util::text::color::NamedColor::Gold => lithummc::plugin::common::NamedColor::Gold,
        lithummc_util::text::color::NamedColor::Gray => lithummc::plugin::common::NamedColor::Gray,
        lithummc_util::text::color::NamedColor::DarkGray => {
            lithummc::plugin::common::NamedColor::DarkGray
        }
        lithummc_util::text::color::NamedColor::Blue => lithummc::plugin::common::NamedColor::Blue,
        lithummc_util::text::color::NamedColor::Green => lithummc::plugin::common::NamedColor::Green,
        lithummc_util::text::color::NamedColor::Aqua => lithummc::plugin::common::NamedColor::Aqua,
        lithummc_util::text::color::NamedColor::Red => lithummc::plugin::common::NamedColor::Red,
        lithummc_util::text::color::NamedColor::LightPurple => {
            lithummc::plugin::common::NamedColor::LightPurple
        }
        lithummc_util::text::color::NamedColor::Yellow => {
            lithummc::plugin::common::NamedColor::Yellow
        }
        lithummc_util::text::color::NamedColor::White => lithummc::plugin::common::NamedColor::White,
    }
}

impl HostBedrockScoreboard for PluginHostState {
    fn add_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
        display_name: String,
        sort_order: scoreboard::BedrockSortOrder,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(
            *custom_guard,
            Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(
                lithummc_core::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.add_objective(
            player.as_ref(),
            lithummc_core::world::scoreboard::BedrockObjective {
                name,
                display_name,
                sort_order: match sort_order {
                    scoreboard::BedrockSortOrder::Ascending => {
                        lithummc_core::world::scoreboard::BedrockSortOrder::Ascending
                    }
                    scoreboard::BedrockSortOrder::Descending => {
                        lithummc_core::world::scoreboard::BedrockSortOrder::Descending
                    }
                },
            },
        );
        Ok(())
    }

    fn update_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
        display_name: String,
        sort_order: scoreboard::BedrockSortOrder,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(
            *custom_guard,
            Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(
                lithummc_core::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.update_objective(
            player.as_ref(),
            lithummc_core::world::scoreboard::BedrockObjective {
                name,
                display_name,
                sort_order: match sort_order {
                    scoreboard::BedrockSortOrder::Ascending => {
                        lithummc_core::world::scoreboard::BedrockSortOrder::Ascending
                    }
                    scoreboard::BedrockSortOrder::Descending => {
                        lithummc_core::world::scoreboard::BedrockSortOrder::Descending
                    }
                },
            },
        );
        Ok(())
    }

    fn remove_objective(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        {
            sb.remove_objective(player.as_ref(), &name);
        }
        Ok(())
    }

    fn set_display_slot(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        slot: scoreboard::BedrockDisplaySlot,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(
            *custom_guard,
            Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(
                lithummc_core::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        let b_slot = match slot {
            scoreboard::BedrockDisplaySlot::PlayerList => {
                lithummc_core::world::scoreboard::BedrockDisplaySlot::PlayerList
            }
            scoreboard::BedrockDisplaySlot::Sidebar => {
                lithummc_core::world::scoreboard::BedrockDisplaySlot::Sidebar
            }
            scoreboard::BedrockDisplaySlot::BelowName => {
                lithummc_core::world::scoreboard::BedrockDisplaySlot::BelowName
            }
        };
        sb.set_display_objective(player.as_ref(), b_slot, Some(&objective_name));
        Ok(())
    }

    fn clear_display_slot(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        slot: scoreboard::BedrockDisplaySlot,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        {
            let b_slot = match slot {
                scoreboard::BedrockDisplaySlot::PlayerList => {
                    lithummc_core::world::scoreboard::BedrockDisplaySlot::PlayerList
                }
                scoreboard::BedrockDisplaySlot::Sidebar => {
                    lithummc_core::world::scoreboard::BedrockDisplaySlot::Sidebar
                }
                scoreboard::BedrockDisplaySlot::BelowName => {
                    lithummc_core::world::scoreboard::BedrockDisplaySlot::BelowName
                }
            };
            sb.clear_display_objective(player.as_ref(), b_slot);
        }
        Ok(())
    }

    fn update_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
        value: i32,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(
            *custom_guard,
            Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(
                lithummc_core::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        sb.update_score(player.as_ref(), &entity_name, &objective_name, value);
        Ok(())
    }

    fn add_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
        delta: i32,
    ) -> wasmtime::Result<i32> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !matches!(
            *custom_guard,
            Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(_))
        ) {
            *custom_guard = Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(
                lithummc_core::world::scoreboard::BedrockScoreboard::default(),
            ));
        }
        let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        else {
            return Err(wasmtime::Error::msg("Invalid scoreboard state"));
        };
        let new_val = sb.add_score(player.as_ref(), entity_name, objective_name, delta);
        Ok(new_val)
    }

    fn remove_score(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
        objective_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        {
            sb.remove_score(player.as_ref(), &entity_name, &objective_name);
        }
        Ok(())
    }

    fn reset_entity_scores(
        &mut self,
        res: Resource<scoreboard::BedrockScoreboard>,
        entity_name: String,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?.clone();
        let mut custom_guard = player
            .custom_scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(lithummc_core::entity::player::CustomScoreboard::Bedrock(sb)) =
            custom_guard.as_mut()
        {
            sb.reset_scores_for_entity(player.as_ref(), &entity_name);
        }
        Ok(())
    }

    fn drop(&mut self, _res: Resource<scoreboard::BedrockScoreboard>) -> wasmtime::Result<()> {
        Ok(())
    }
}
