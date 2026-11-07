use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;

pub use lithummc_command::node::*;

pub mod attached {
    pub use lithummc_command::node::attached::*;
    pub type AttachedNode =
        lithummc_command::node::attached::AttachedNode<crate::command::CommandSource>;
    pub type RootAttachedNode =
        lithummc_command::node::attached::RootAttachedNode<crate::command::CommandSource>;
    pub type LiteralAttachedNode =
        lithummc_command::node::attached::LiteralAttachedNode<crate::command::CommandSource>;
    pub type CommandAttachedNode =
        lithummc_command::node::attached::CommandAttachedNode<crate::command::CommandSource>;
    pub type ArgumentAttachedNode =
        lithummc_command::node::attached::ArgumentAttachedNode<crate::command::CommandSource>;
}

pub mod detached {
    pub use lithummc_command::node::detached::*;
    pub type DetachedNode =
        lithummc_command::node::detached::DetachedNode<crate::command::CommandSource>;
    pub type LiteralDetachedNode =
        lithummc_command::node::detached::LiteralDetachedNode<crate::command::CommandSource>;
    pub type CommandDetachedNode =
        lithummc_command::node::detached::CommandDetachedNode<crate::command::CommandSource>;
    pub type ArgumentDetachedNode =
        lithummc_command::node::detached::ArgumentDetachedNode<crate::command::CommandSource>;
}

pub mod tree {
    pub use lithummc_command::node::tree::*;
    pub type Tree = lithummc_command::node::tree::Tree<crate::command::CommandSource>;
}

pub mod dispatcher {
    pub use lithummc_command::dispatcher::*;
    pub type CommandDispatcher =
        lithummc_command::dispatcher::CommandDispatcher<crate::command::CommandSource>;
}

pub type Command = lithummc_command::node::Command<CommandSource>;
pub type Requirement = lithummc_command::node::Requirement<CommandSource>;
pub type Requirements = lithummc_command::node::Requirements<CommandSource>;
pub type RedirectModifier = lithummc_command::node::RedirectModifier<CommandSource>;
pub type RedirectModifierResult = lithummc_command::node::RedirectModifierResult<CommandSource>;
pub type RedirectModifierExecutor = lithummc_command::node::RedirectModifierExecutor<CommandSource>;

/// A struct implementing this trait is able to run with a given context.
pub trait CommandExecutor: Sync + Send {
    /// Executes this executor for a command.
    fn execute(&self, context: &CommandContext) -> lithummc_command::node::CommandExecutorResult;
}

pub struct CommandExecutorAdapter<T>(pub T);

impl<T: CommandExecutor> lithummc_command::node::CommandExecutor<CommandSource>
    for CommandExecutorAdapter<T>
{
    fn execute(
        &self,
        context: &lithummc_command::context::command_context::CommandContext<'_, CommandSource>,
    ) -> lithummc_command::node::CommandExecutorResult {
        self.0.execute(context)
    }
}

pub struct ArcCommandExecutorAdapter(pub std::sync::Arc<dyn CommandExecutor>);

impl lithummc_command::node::CommandExecutor<CommandSource> for ArcCommandExecutorAdapter {
    fn execute(
        &self,
        context: &lithummc_command::context::command_context::CommandContext<'_, CommandSource>,
    ) -> lithummc_command::node::CommandExecutorResult {
        self.0.execute(context)
    }
}
