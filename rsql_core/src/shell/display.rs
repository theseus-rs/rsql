use crate::configuration::ResultFormat;
use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "display"
    }

    fn args(&self) -> &'static str {
        "ascii|unicode"
    }

    fn description(&self) -> &'static str {
        "Display results in ASCII or Unicode"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let results_display = match options.input[1].to_lowercase().as_str() {
            "ascii" => ResultFormat::Ascii,
            "unicode" => ResultFormat::Unicode,
            option => bail!("Invalid display mode option: {option}"),
        };

        options.configuration.results_format = results_display;

        Ok(LoopCondition::Continue)
    }
}
