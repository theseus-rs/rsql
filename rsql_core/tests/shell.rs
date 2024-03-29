use indoc::indoc;
use rsql_core::configuration::Configuration;
use rsql_core::shell::{ShellArgs, ShellBuilder};

#[tokio::test]
async fn test_execute_command() -> anyhow::Result<()> {
    let configuration = Configuration::default();
    let commands = vec![".locale en".to_string(), ".locale".to_string()];
    let args = ShellArgs {
        commands,
        ..Default::default()
    };
    let mut shell = ShellBuilder::default()
        .with_configuration(configuration)
        .build();
    let _ = shell.execute(&args).await?;

    let command_output = shell.output.to_string();
    let expected = indoc! {r#"
            Locale: en
        "#};
    assert_eq!(command_output, expected);
    Ok(())
}
