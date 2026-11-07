use wasmtime::component::{Access, HasSelf, Resource};

use crate::{
    commands::executor::{WasmCommandExecutor, WasmCommandSuggestionProvider},
    lithummc::{
        self,
        plugin::{
            command::{
                Arg, ArgumentType, Command, CommandNode, CommandSender, CommandSenderType,
                ConsumedArgs, PermissionLevel, StringType,
            },
            common::{BlockPos as WitBlockPos, Locale, Position},
            player::Player,
            server::Server,
            text::TextComponent,
            world::World,
        },
    },
};
use lithummc_core::command::{
    argument_builder::{argument, literal},
    argument_types::{
        block::BlockArgumentType,
        block_predicate::BlockPredicateArgumentType,
        component::ComponentArgumentType,
        coordinates::{
            block_pos::BlockPosArgumentType, rotation::RotationArgumentType,
            vec2::Vec2ArgumentType, vec3::Vec3ArgumentType,
        },
        core::{
            bool::BoolArgumentType, double::DoubleArgumentType, float::FloatArgumentType,
            integer::IntegerArgumentType, long::LongArgumentType, string::StringArgumentType,
        },
        entity::EntityArgumentType,
        entity_anchor::EntityAnchorArgumentType,
        game_profile::GameProfileArgumentType,
        gamemode::GameModeArgumentType,
        identifier::IdentifierArgumentType,
        item::ItemStackArgumentType,
        item_predicate::ItemPredicateArgumentType,
        time::TimeArgumentType,
    },
};
use lithummc_wasm_host_common::state::{PluginHostState, WasmCommand, WasmCommandNode};

pub mod executor;

impl lithummc::plugin::command::Host for PluginHostState {}

impl lithummc::plugin::command::HostConsumedArgs for PluginHostState {
    #[expect(clippy::too_many_lines)]
    fn get_value(
        &mut self,
        consumed_args: Resource<ConsumedArgs>,
        key: String,
    ) -> wasmtime::Result<Arg> {
        use lithummc_wasm_host_common::args::OwnedArg;

        let resource = self.get(&consumed_args)?;

        let Some(owned_arg) = resource.get(&key).cloned() else {
            return Ok(Arg::Simple(String::new()));
        };

        Ok(match owned_arg {
            OwnedArg::Simple(s) => Arg::Simple(s),
            OwnedArg::Msg(s) => Arg::Msg(s),
            OwnedArg::Bool(b) => Arg::Bool(b),
            OwnedArg::Item(s) => Arg::Item(s),
            OwnedArg::ItemPredicate(s) => Arg::ItemPredicate(s),
            OwnedArg::ResourceLocation(s) => Arg::ResourceLocation(s),
            OwnedArg::Block(s) => Arg::Block(s),
            OwnedArg::BlockPredicate(s) => Arg::BlockPredicate(s),
            OwnedArg::Time(t) => Arg::Time(t),
            OwnedArg::Num(n) => {
                use lithummc_wasm_host_common::args::{NotInBounds, Number};
                let convert_num = |n: Number| match n {
                    Number::F64(v) => lithummc::plugin::command::Number::Float64(v),
                    Number::F32(v) => lithummc::plugin::command::Number::Float32(v),
                    Number::I32(v) => lithummc::plugin::command::Number::Int32(v),
                    Number::I64(v) => lithummc::plugin::command::Number::Int64(v),
                };
                Arg::Num(n.map(convert_num).map_err(|e| match e {
                    NotInBounds::LowerBound(a, b) => {
                        lithummc::plugin::command::NotInBounds::LowerBound((
                            convert_num(a),
                            convert_num(b),
                        ))
                    }
                    NotInBounds::UpperBound(a, b) => {
                        lithummc::plugin::command::NotInBounds::UpperBound((
                            convert_num(a),
                            convert_num(b),
                        ))
                    }
                }))
            }
            OwnedArg::BlockPos(p) => Arg::BlockPos(WitBlockPos {
                x: p.0.x,
                y: p.0.y,
                z: p.0.z,
            }),
            OwnedArg::Pos3D(v) => Arg::Pos3d((v.x, v.y, v.z)),
            OwnedArg::Pos2D(v) => Arg::Pos2d((v.x, v.y)),
            OwnedArg::Rotation(a, b, c, d) => Arg::Rotation((a, b, c, d)),
            OwnedArg::GameMode(g) => Arg::Gamemode(match g {
                lithummc_util::GameMode::Survival => lithummc::plugin::common::GameMode::Survival,
                lithummc_util::GameMode::Creative => lithummc::plugin::common::GameMode::Creative,
                lithummc_util::GameMode::Adventure => lithummc::plugin::common::GameMode::Adventure,
                lithummc_util::GameMode::Spectator => lithummc::plugin::common::GameMode::Spectator,
            }),
            OwnedArg::Difficulty(d) => Arg::Difficulty(match d {
                lithummc_util::Difficulty::Peaceful => lithummc::plugin::server::Difficulty::Peaceful,
                lithummc_util::Difficulty::Easy => lithummc::plugin::server::Difficulty::Easy,
                lithummc_util::Difficulty::Normal => lithummc::plugin::server::Difficulty::Normal,
                lithummc_util::Difficulty::Hard => lithummc::plugin::server::Difficulty::Hard,
            }),
            OwnedArg::Players(players) => {
                let mut resources = Vec::new();
                for p in players {
                    if let Ok(r) = self.add(p) {
                        resources.push(r);
                    }
                }
                Arg::Players(resources)
            }
            OwnedArg::Particle(p) => Arg::Particle(format!("{p:?}")),
            OwnedArg::TextComponent(t) => Arg::TextComponent(self.add(t)?),
            OwnedArg::BossbarColor(c) => Arg::BossbarColor(match c {
                lithummc_core::world::bossbar::BossbarColor::Pink => {
                    lithummc::plugin::command::BossbarColor::Pink
                }
                lithummc_core::world::bossbar::BossbarColor::Blue => {
                    lithummc::plugin::command::BossbarColor::Blue
                }
                lithummc_core::world::bossbar::BossbarColor::Red => {
                    lithummc::plugin::command::BossbarColor::Red
                }
                lithummc_core::world::bossbar::BossbarColor::Green => {
                    lithummc::plugin::command::BossbarColor::Green
                }
                lithummc_core::world::bossbar::BossbarColor::Yellow => {
                    lithummc::plugin::command::BossbarColor::Yellow
                }
                lithummc_core::world::bossbar::BossbarColor::Purple => {
                    lithummc::plugin::command::BossbarColor::Purple
                }
                lithummc_core::world::bossbar::BossbarColor::White => {
                    lithummc::plugin::command::BossbarColor::White
                }
            }),
            OwnedArg::BossbarStyle(s) => Arg::BossbarStyle(match s {
                lithummc_core::world::bossbar::BossbarDivisions::NoDivision => {
                    lithummc::plugin::command::BossbarStyle::NoDivision
                }
                lithummc_core::world::bossbar::BossbarDivisions::Notches6 => {
                    lithummc::plugin::command::BossbarStyle::Notches6
                }
                lithummc_core::world::bossbar::BossbarDivisions::Notches10 => {
                    lithummc::plugin::command::BossbarStyle::Notches10
                }
                lithummc_core::world::bossbar::BossbarDivisions::Notches12 => {
                    lithummc::plugin::command::BossbarStyle::Notches12
                }
                lithummc_core::world::bossbar::BossbarDivisions::Notches20 => {
                    lithummc::plugin::command::BossbarStyle::Notches20
                }
            }),
            OwnedArg::SoundCategory(s) => Arg::SoundCategory(match s {
                lithummc_data::sound::SoundCategory::Master
                | lithummc_data::sound::SoundCategory::Ui => {
                    lithummc::plugin::command::SoundCategory::Master
                }
                lithummc_data::sound::SoundCategory::Music => {
                    lithummc::plugin::command::SoundCategory::Music
                }
                lithummc_data::sound::SoundCategory::Records => {
                    lithummc::plugin::command::SoundCategory::Records
                }
                lithummc_data::sound::SoundCategory::Weather => {
                    lithummc::plugin::command::SoundCategory::Weather
                }
                lithummc_data::sound::SoundCategory::Blocks => {
                    lithummc::plugin::command::SoundCategory::Blocks
                }
                lithummc_data::sound::SoundCategory::Hostile => {
                    lithummc::plugin::command::SoundCategory::Hostile
                }
                lithummc_data::sound::SoundCategory::Neutral => {
                    lithummc::plugin::command::SoundCategory::Neutral
                }
                lithummc_data::sound::SoundCategory::Players => {
                    lithummc::plugin::command::SoundCategory::Players
                }
                lithummc_data::sound::SoundCategory::Ambient => {
                    lithummc::plugin::command::SoundCategory::Ambient
                }
                lithummc_data::sound::SoundCategory::Voice => {
                    lithummc::plugin::command::SoundCategory::Voice
                }
            }),
            OwnedArg::DamageType(d) => Arg::DamageType(d.message_id.to_string()),
            OwnedArg::Effect(e) => Arg::Effect(e.minecraft_name.to_string()),
            OwnedArg::Enchantment(e) => Arg::Enchantment(e.name.to_string()),
            OwnedArg::Advancement(a) => Arg::Advancement(a.to_string()),
            OwnedArg::EntityAnchor(a) => Arg::EntityAnchor(match a {
                lithummc_core::command::argument_types::entity_anchor::EntityAnchor::Eyes => {
                    lithummc::plugin::command::EntityAnchor::Eyes
                }
                lithummc_core::command::argument_types::entity_anchor::EntityAnchor::Feet => {
                    lithummc::plugin::command::EntityAnchor::Feet
                }
            }),
            // These types don't have direct WIT resource mappings yet
            OwnedArg::Entities(_) | OwnedArg::Entity(_) | OwnedArg::GameProfiles(_) => {
                Arg::Simple(String::new())
            }
        })
    }

    fn drop(&mut self, rep: Resource<ConsumedArgs>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::command::HostCommand for PluginHostState {
    fn new(
        &mut self,
        names: Vec<String>,
        description: String,
    ) -> wasmtime::Result<Resource<Command>> {
        self.add(WasmCommand::new(names, description))
            .map_err(|_| wasmtime::Error::msg("Failed to add command resource"))
    }

    fn then(
        &mut self,
        command: Resource<Command>,
        node: Resource<CommandNode>,
    ) -> wasmtime::Result<()> {
        let node_data = self.take(node)?;
        let command_res = self.get_mut(&command)?;
        let cmd = std::mem::replace(command_res, WasmCommand::new(Vec::new(), String::new()));
        *command_res = cmd.then(node_data);
        Ok(())
    }

    fn execute_with_handler_id(
        &mut self,
        command: Resource<Command>,
        handler_id: u32,
    ) -> wasmtime::Result<()> {
        let plugin = self
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin dropped"))?;
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not initialized"))?;

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };
        let command_res = self.get_mut(&command)?;
        let cmd = std::mem::replace(command_res, WasmCommand::new(Vec::new(), String::new()));
        *command_res = cmd.executes(executor);
        Ok(())
    }

    fn drop(&mut self, rep: Resource<Command>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::command::HostCommandSender for PluginHostState {
    fn get_command_sender_type(
        &mut self,
        res: Resource<CommandSender>,
    ) -> wasmtime::Result<CommandSenderType> {
        let sender = self.get(&res)?.clone();
        match sender {
            lithummc_core::command::CommandSender::Console => Ok(CommandSenderType::Console),
            lithummc_core::command::CommandSender::Player(player) => {
                Ok(CommandSenderType::Player(self.add(player)?))
            }
            lithummc_core::command::CommandSender::CommandBlock(block_entity, world) => Ok(
                CommandSenderType::CommandBlock((self.add(block_entity)?, self.add(world)?)),
            ),
            lithummc_core::command::CommandSender::Dummy => Ok(CommandSenderType::Dummy),
        }
    }

    fn get_name(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<String> {
        Ok(self.get(&sender)?.to_string())
    }

    fn send_message(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        self.get(&sender)?.send_message(component);
        Ok(())
    }

    fn send_system_message(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        self.get(&sender)?.send_message(component);
        Ok(())
    }

    fn send_error(
        &mut self,
        sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = self.take(text)?;
        self.get(&sender)?
            .send_message(component.color(lithummc_util::text::color::Color::Named(
                lithummc_util::text::color::NamedColor::Red,
            )));
        Ok(())
    }

    fn set_success_count(
        &mut self,
        sender: Resource<CommandSender>,
        count: i32,
    ) -> wasmtime::Result<()> {
        self.get_mut(&sender)?.set_success_count(count as u32);
        Ok(())
    }

    fn is_player(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<bool> {
        Ok(matches!(
            self.get(&sender)?,
            lithummc_core::command::CommandSender::Player(_)
        ))
    }

    fn is_console(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<bool> {
        Ok(matches!(
            self.get(&sender)?,
            lithummc_core::command::CommandSender::Console
        ))
    }

    fn as_player(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        if let lithummc_core::command::CommandSender::Player(player) = &self.get(&sender)? {
            Ok(Some(self.add(player.clone()).map_err(|_| {
                wasmtime::Error::msg("Failed to add player resource")
            })?))
        } else {
            Ok(None)
        }
    }

    fn permission_level(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<PermissionLevel> {
        Ok(match self.get(&sender)?.permission_lvl() {
            lithummc_util::PermissionLvl::Zero => PermissionLevel::Zero,
            lithummc_util::PermissionLvl::One => PermissionLevel::One,
            lithummc_util::PermissionLvl::Two => PermissionLevel::Two,
            lithummc_util::PermissionLvl::Three => PermissionLevel::Three,
            lithummc_util::PermissionLvl::Four => PermissionLevel::Four,
        })
    }

    fn has_permission_level(
        &mut self,
        sender: Resource<CommandSender>,
        level: PermissionLevel,
    ) -> wasmtime::Result<bool> {
        let required = match level {
            PermissionLevel::Zero => lithummc_util::PermissionLvl::Zero,
            PermissionLevel::One => lithummc_util::PermissionLvl::One,
            PermissionLevel::Two => lithummc_util::PermissionLvl::Two,
            PermissionLevel::Three => lithummc_util::PermissionLvl::Three,
            PermissionLevel::Four => lithummc_util::PermissionLvl::Four,
        };
        Ok(self.get(&sender)?.permission_lvl() >= required)
    }

    fn position(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<Option<Position>> {
        Ok(self.get(&sender)?.position().map(|p| (p.x, p.y, p.z)))
    }

    fn world(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<Option<Resource<World>>> {
        if let Some(world) = self.get(&sender)?.world() {
            Ok(Some(self.add(world).map_err(|_| {
                wasmtime::Error::msg("Failed to add world resource")
            })?))
        } else {
            Ok(None)
        }
    }

    fn get_locale(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<Locale> {
        Ok(map_util_locale_to_wit(self.get(&sender)?.get_locale()))
    }

    fn should_receive_feedback(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<bool> {
        Ok(self.get(&sender)?.should_receive_feedback())
    }

    fn should_broadcast_console_to_ops(
        &mut self,
        sender: Resource<CommandSender>,
    ) -> wasmtime::Result<bool> {
        Ok(self.get(&sender)?.should_broadcast_console_to_ops())
    }

    fn should_track_output(&mut self, sender: Resource<CommandSender>) -> wasmtime::Result<bool> {
        Ok(self.get(&sender)?.should_track_output())
    }

    fn drop(&mut self, rep: Resource<CommandSender>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl lithummc::plugin::command::HostCommandSenderWithStore<PluginHostState>
    for HasSelf<PluginHostState>
{
    async fn has_permission(
        mut host: Access<'_, PluginHostState, Self>,
        sender: Resource<CommandSender>,
        server: /* borrow */ Resource<Server>,
        node: String,
    ) -> wasmtime::Result<bool> {
        let (sender, server, plugin) = {
            let state = host.get();
            let sender = state.get(&sender)?.clone();
            let server = state.get(&server)?.clone();
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (sender, server, plugin)
        };

        plugin
            .store
            .pump_blocking(&mut host, move || sender.has_permission(&server, &node))
            .await
    }
}

impl lithummc::plugin::command::HostCommandNode for PluginHostState {
    fn literal(&mut self, name: String) -> wasmtime::Result<Resource<CommandNode>> {
        self.add(WasmCommandNode::Literal(literal(name)))
            .map_err(|_| wasmtime::Error::msg("Failed to add literal node"))
    }

    fn argument(
        &mut self,
        name: String,
        arg_type: ArgumentType,
    ) -> wasmtime::Result<Resource<CommandNode>> {
        let node = match arg_type {
            ArgumentType::Bool => WasmCommandNode::Argument(argument(name, BoolArgumentType)),
            ArgumentType::Float((min, max)) => WasmCommandNode::Argument(argument(
                name,
                FloatArgumentType::new(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX)),
            )),
            ArgumentType::Double((min, max)) => WasmCommandNode::Argument(argument(
                name,
                DoubleArgumentType::new(min.unwrap_or(f64::MIN), max.unwrap_or(f64::MAX)),
            )),
            ArgumentType::Integer((min, max)) => WasmCommandNode::Argument(argument(
                name,
                IntegerArgumentType::new(min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX)),
            )),
            ArgumentType::Long((min, max)) => WasmCommandNode::Argument(argument(
                name,
                LongArgumentType::new(min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX)),
            )),
            ArgumentType::String(st) => match st {
                StringType::SingleWord => {
                    WasmCommandNode::Argument(argument(name, StringArgumentType::SingleWord))
                }
                StringType::Quotable => {
                    WasmCommandNode::Argument(argument(name, StringArgumentType::QuotablePhrase))
                }
                StringType::Greedy => {
                    WasmCommandNode::Argument(argument(name, StringArgumentType::GreedyPhrase))
                }
            },
            ArgumentType::Entities => {
                WasmCommandNode::Argument(argument(name, EntityArgumentType::Entities))
            }
            ArgumentType::Entity => {
                WasmCommandNode::Argument(argument(name, EntityArgumentType::Entity))
            }
            ArgumentType::Players => {
                WasmCommandNode::Argument(argument(name, EntityArgumentType::Players))
            }
            ArgumentType::GameProfile => {
                WasmCommandNode::Argument(argument(name, GameProfileArgumentType))
            }
            ArgumentType::BlockPos => {
                WasmCommandNode::Argument(argument(name, BlockPosArgumentType))
            }
            ArgumentType::Position3d => {
                WasmCommandNode::Argument(argument(name, Vec3ArgumentType::Default))
            }
            ArgumentType::Position2d => {
                WasmCommandNode::Argument(argument(name, Vec2ArgumentType::Default))
            }
            ArgumentType::BlockState => {
                WasmCommandNode::Argument(argument(name, BlockArgumentType))
            }
            ArgumentType::BlockPredicate => {
                WasmCommandNode::Argument(argument(name, BlockPredicateArgumentType))
            }
            ArgumentType::Item => WasmCommandNode::Argument(argument(name, ItemStackArgumentType)),
            ArgumentType::ItemPredicate => {
                WasmCommandNode::Argument(argument(name, ItemPredicateArgumentType))
            }
            ArgumentType::Component => {
                WasmCommandNode::Argument(argument(name, ComponentArgumentType))
            }
            ArgumentType::Rotation => {
                WasmCommandNode::Argument(argument(name, RotationArgumentType))
            }
            ArgumentType::ResourceLocation | ArgumentType::Resource(_) => {
                WasmCommandNode::Argument(argument(name, IdentifierArgumentType))
            }
            ArgumentType::EntityAnchor => {
                WasmCommandNode::Argument(argument(name, EntityAnchorArgumentType))
            }
            ArgumentType::Gamemode => {
                WasmCommandNode::Argument(argument(name, GameModeArgumentType))
            }
            ArgumentType::Difficulty => {
                WasmCommandNode::Argument(argument(name, StringArgumentType::SingleWord))
            }
            ArgumentType::Time(min) => {
                WasmCommandNode::Argument(argument(name, TimeArgumentType::new(min.unwrap_or(0))))
            }
            _ => {
                return Err(wasmtime::Error::msg(format!(
                    "Unimplemented argument type: {arg_type:?}"
                )));
            }
        };
        self.add(node)
            .map_err(|_| wasmtime::Error::msg("Failed to add argument node"))
    }

    fn then(
        &mut self,
        self_node: Resource<CommandNode>,
        node: Resource<CommandNode>,
    ) -> wasmtime::Result<()> {
        let child = self.take(node)?;
        let parent = self.get_mut(&self_node)?;
        let builder = std::mem::replace(parent, WasmCommandNode::Literal(literal("")));
        *parent = builder.then(child);
        Ok(())
    }

    fn execute_with_handler_id(
        &mut self,
        node: Resource<CommandNode>,
        handler_id: u32,
    ) -> wasmtime::Result<()> {
        let plugin = self
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin dropped"))?;
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not initialized"))?;

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };
        let resource = self.get_mut(&node)?;
        let builder = std::mem::replace(resource, WasmCommandNode::Literal(literal("")));
        *resource = builder.executes(executor);
        Ok(())
    }

    fn suggest_with_handler_id(
        &mut self,
        node: Resource<CommandNode>,
        handler_id: u32,
    ) -> wasmtime::Result<()> {
        let plugin = self
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin dropped"))?;
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not initialized"))?;

        let provider = WasmCommandSuggestionProvider {
            handler_id,
            plugin,
            server,
        };
        let resource = self.get_mut(&node)?;
        let builder = std::mem::replace(resource, WasmCommandNode::Literal(literal("")));
        *resource = builder.suggests(provider);
        Ok(())
    }

    fn require_with_handler_id(
        &mut self,
        _node: Resource<CommandNode>,
        _handler_id: u32,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "require_with_handler_id not implemented",
        ))
    }

    fn drop(&mut self, rep: Resource<CommandNode>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

#[expect(clippy::too_many_lines)]
const fn map_util_locale_to_wit(locale: lithummc_util::translation::Locale) -> Locale {
    match locale {
        lithummc_util::translation::Locale::AfZa => Locale::AfZa,
        lithummc_util::translation::Locale::ArSa => Locale::ArSa,
        lithummc_util::translation::Locale::AstEs => Locale::AstEs,
        lithummc_util::translation::Locale::AzAz => Locale::AzAz,
        lithummc_util::translation::Locale::BaRu => Locale::BaRu,
        lithummc_util::translation::Locale::Bar => Locale::Bar,
        lithummc_util::translation::Locale::BeBy => Locale::BeBy,
        lithummc_util::translation::Locale::BgBg => Locale::BgBg,
        lithummc_util::translation::Locale::BrFr => Locale::BrFr,
        lithummc_util::translation::Locale::Brb => Locale::Brb,
        lithummc_util::translation::Locale::BsBa => Locale::BsBa,
        lithummc_util::translation::Locale::CaEs => Locale::CaEs,
        lithummc_util::translation::Locale::CsCz => Locale::CsCz,
        lithummc_util::translation::Locale::CyGb => Locale::CyGb,
        lithummc_util::translation::Locale::DaDk => Locale::DaDk,
        lithummc_util::translation::Locale::DeAt => Locale::DeAt,
        lithummc_util::translation::Locale::DeCh => Locale::DeCh,
        lithummc_util::translation::Locale::DeDe => Locale::DeDe,
        lithummc_util::translation::Locale::ElGr => Locale::ElGr,
        lithummc_util::translation::Locale::EnAu => Locale::EnAu,
        lithummc_util::translation::Locale::EnCa => Locale::EnCa,
        lithummc_util::translation::Locale::EnGb => Locale::EnGb,
        lithummc_util::translation::Locale::EnNz => Locale::EnNz,
        lithummc_util::translation::Locale::EnPt => Locale::EnPt,
        lithummc_util::translation::Locale::EnUd => Locale::EnUd,
        lithummc_util::translation::Locale::EnUs => Locale::EnUs,
        lithummc_util::translation::Locale::Enp => Locale::Enp,
        lithummc_util::translation::Locale::Enws => Locale::Enws,
        lithummc_util::translation::Locale::EoUy => Locale::EoUy,
        lithummc_util::translation::Locale::EsAr => Locale::EsAr,
        lithummc_util::translation::Locale::EsCl => Locale::EsCl,
        lithummc_util::translation::Locale::EsEc => Locale::EsEc,
        lithummc_util::translation::Locale::EsEs => Locale::EsEs,
        lithummc_util::translation::Locale::EsMx => Locale::EsMx,
        lithummc_util::translation::Locale::EsUy => Locale::EsUy,
        lithummc_util::translation::Locale::EsVe => Locale::EsVe,
        lithummc_util::translation::Locale::Esan => Locale::Esan,
        lithummc_util::translation::Locale::EtEe => Locale::EtEe,
        lithummc_util::translation::Locale::EuEs => Locale::EuEs,
        lithummc_util::translation::Locale::FaIr => Locale::FaIr,
        lithummc_util::translation::Locale::FiFi => Locale::FiFi,
        lithummc_util::translation::Locale::FilPh => Locale::FilPh,
        lithummc_util::translation::Locale::FoFo => Locale::FoFo,
        lithummc_util::translation::Locale::FrCa => Locale::FrCa,
        lithummc_util::translation::Locale::FrFr => Locale::FrFr,
        lithummc_util::translation::Locale::FraDe => Locale::FraDe,
        lithummc_util::translation::Locale::FurIt => Locale::FurIt,
        lithummc_util::translation::Locale::FyNl => Locale::FyNl,
        lithummc_util::translation::Locale::GaIe => Locale::GaIe,
        lithummc_util::translation::Locale::GdGb => Locale::GdGb,
        lithummc_util::translation::Locale::GlEs => Locale::GlEs,
        lithummc_util::translation::Locale::HawUs => Locale::HawUs,
        lithummc_util::translation::Locale::HeIl => Locale::HeIl,
        lithummc_util::translation::Locale::HiIn => Locale::HiIn,
        lithummc_util::translation::Locale::HrHr => Locale::HrHr,
        lithummc_util::translation::Locale::HuHu => Locale::HuHu,
        lithummc_util::translation::Locale::HyAm => Locale::HyAm,
        lithummc_util::translation::Locale::IdId => Locale::IdId,
        lithummc_util::translation::Locale::IgNg => Locale::IgNg,
        lithummc_util::translation::Locale::IoEn => Locale::IoEn,
        lithummc_util::translation::Locale::IsIs => Locale::IsIs,
        lithummc_util::translation::Locale::Isv => Locale::Isv,
        lithummc_util::translation::Locale::ItIt => Locale::ItIt,
        lithummc_util::translation::Locale::JaJp => Locale::JaJp,
        lithummc_util::translation::Locale::JboEn => Locale::JboEn,
        lithummc_util::translation::Locale::KaGe => Locale::KaGe,
        lithummc_util::translation::Locale::KkKz => Locale::KkKz,
        lithummc_util::translation::Locale::KnIn => Locale::KnIn,
        lithummc_util::translation::Locale::KoKr => Locale::KoKr,
        lithummc_util::translation::Locale::Ksh => Locale::Ksh,
        lithummc_util::translation::Locale::KwGb => Locale::KwGb,
        lithummc_util::translation::Locale::LaLa => Locale::LaLa,
        lithummc_util::translation::Locale::LbLu => Locale::LbLu,
        lithummc_util::translation::Locale::LiLi => Locale::LiLi,
        lithummc_util::translation::Locale::Lmo => Locale::Lmo,
        lithummc_util::translation::Locale::LoLa => Locale::LoLa,
        lithummc_util::translation::Locale::LolUs => Locale::LolUs,
        lithummc_util::translation::Locale::LtLt => Locale::LtLt,
        lithummc_util::translation::Locale::LvLv => Locale::LvLv,
        lithummc_util::translation::Locale::Lzh => Locale::Lzh,
        lithummc_util::translation::Locale::MkMk => Locale::MkMk,
        lithummc_util::translation::Locale::MnMn => Locale::MnMn,
        lithummc_util::translation::Locale::MsMy => Locale::MsMy,
        lithummc_util::translation::Locale::MtMt => Locale::MtMt,
        lithummc_util::translation::Locale::Nah => Locale::Nah,
        lithummc_util::translation::Locale::NdsDe => Locale::NdsDe,
        lithummc_util::translation::Locale::NlBe => Locale::NlBe,
        lithummc_util::translation::Locale::NlNl => Locale::NlNl,
        lithummc_util::translation::Locale::NnNo => Locale::NnNo,
        lithummc_util::translation::Locale::NoNo => Locale::NoNo,
        lithummc_util::translation::Locale::OcFr => Locale::OcFr,
        lithummc_util::translation::Locale::Ovd => Locale::Ovd,
        lithummc_util::translation::Locale::PlPl => Locale::PlPl,
        lithummc_util::translation::Locale::PtBr => Locale::PtBr,
        lithummc_util::translation::Locale::PtPt => Locale::PtPt,
        lithummc_util::translation::Locale::QyaAa => Locale::QyaAa,
        lithummc_util::translation::Locale::RoRo => Locale::RoRo,
        lithummc_util::translation::Locale::Rpr => Locale::Rpr,
        lithummc_util::translation::Locale::RuRu => Locale::RuRu,
        lithummc_util::translation::Locale::RyUa => Locale::RyUa,
        lithummc_util::translation::Locale::SahSah => Locale::SahSah,
        lithummc_util::translation::Locale::SeNo => Locale::SeNo,
        lithummc_util::translation::Locale::SkSk => Locale::SkSk,
        lithummc_util::translation::Locale::SlSi => Locale::SlSi,
        lithummc_util::translation::Locale::SoSo => Locale::SoSo,
        lithummc_util::translation::Locale::SqAl => Locale::SqAl,
        lithummc_util::translation::Locale::SrCs => Locale::SrCs,
        lithummc_util::translation::Locale::SrSp => Locale::SrSp,
        lithummc_util::translation::Locale::SvSe => Locale::SvSe,
        lithummc_util::translation::Locale::Sxu => Locale::Sxu,
        lithummc_util::translation::Locale::Szl => Locale::Szl,
        lithummc_util::translation::Locale::TaIn => Locale::TaIn,
        lithummc_util::translation::Locale::ThTh => Locale::ThTh,
        lithummc_util::translation::Locale::TlPh => Locale::TlPh,
        lithummc_util::translation::Locale::TlhAa => Locale::TlhAa,
        lithummc_util::translation::Locale::Tok => Locale::Tok,
        lithummc_util::translation::Locale::TrTr => Locale::TrTr,
        lithummc_util::translation::Locale::TtRu => Locale::TtRu,
        lithummc_util::translation::Locale::UkUa => Locale::UkUa,
        lithummc_util::translation::Locale::ValEs => Locale::ValEs,
        lithummc_util::translation::Locale::VecIt => Locale::VecIt,
        lithummc_util::translation::Locale::ViVn => Locale::ViVn,
        lithummc_util::translation::Locale::YiDe => Locale::YiDe,
        lithummc_util::translation::Locale::YoNg => Locale::YoNg,
        lithummc_util::translation::Locale::ZhCn => Locale::ZhCn,
        lithummc_util::translation::Locale::ZhHk => Locale::ZhHk,
        lithummc_util::translation::Locale::ZhTw => Locale::ZhTw,
        lithummc_util::translation::Locale::ZlmArab => Locale::ZlmArab,
    }
}
