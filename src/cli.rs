#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub logout: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Run(Args),
    Help,
    Version,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: i32,
    pub message: String,
}

#[must_use]
pub fn help_text(bin_name: &str) -> String {
    format!(
        "Longbridge Terminal\n\n用法：\n  {bin_name} [选项]\n\n选项：\n  -h, --help       显示帮助信息\n  -V, --version    显示版本信息\n      --logout     清理本地登录状态（预留）\n"
    )
}

#[must_use]
pub fn version_text() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

pub fn parse_args<I, S>(args: I) -> Result<Command, ParseError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut parsed = Args::default();
    let mut show_help = false;
    let mut show_version = false;

    for raw in args {
        let arg = raw.into();
        match arg.as_str() {
            "-h" | "--help" => show_help = true,
            "-V" | "--version" => show_version = true,
            "--logout" => parsed.logout = true,
            _ if arg.starts_with('-') => {
                return Err(ParseError {
                    code: 2,
                    message: format!("未知选项：{arg}\n\n{}", help_text("longbridge")),
                });
            }
            _ => {
                return Err(ParseError {
                    code: 2,
                    message: format!("不支持的位置参数：{arg}\n\n{}", help_text("longbridge")),
                });
            }
        }
    }

    if show_help {
        return Ok(Command::Help);
    }

    if show_version {
        return Ok(Command::Version);
    }

    Ok(Command::Run(parsed))
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command};

    #[test]
    fn parses_default_run_command() {
        let result = parse_args(Vec::<String>::new());
        assert!(matches!(result, Ok(Command::Run(_))));
    }

    #[test]
    fn parses_help_command() {
        let result = parse_args(["--help"]);
        assert_eq!(result, Ok(Command::Help));
    }

    #[test]
    fn parses_version_command() {
        let result = parse_args(["--version"]);
        assert_eq!(result, Ok(Command::Version));
    }

    #[test]
    fn parses_logout_flag() {
        let result = parse_args(["--logout"]);
        match result {
            Ok(Command::Run(args)) => assert!(args.logout),
            _ => panic!("expected run command with logout flag"),
        }
    }

    #[test]
    fn fails_on_unknown_option() {
        let result = parse_args(["--unknown"]);
        let err = result.expect_err("expected parse error");
        assert_eq!(err.code, 2);
        assert!(err.message.contains("未知选项"));
    }

    #[test]
    fn fails_on_positional_argument() {
        let result = parse_args(["abc"]);
        let err = result.expect_err("expected parse error");
        assert_eq!(err.code, 2);
        assert!(err.message.contains("不支持的位置参数"));
    }
}
