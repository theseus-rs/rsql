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
    let mut output = Vec::new();

    let mut shell = ShellBuilder::new(&mut output)
        .with_configuration(configuration)
        .build();
    let _ = shell.execute(&args).await?;

    let comand_output = String::from_utf8(output)?;
    let expected = indoc! {r#"
            Locale: en
        "#};
    assert_eq!(comand_output, expected);
    Ok(())
}
