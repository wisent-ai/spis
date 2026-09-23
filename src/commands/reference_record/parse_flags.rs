use super::*;

/// Minimal argparse stand-in: `--flag value`, `--flag=value`, `store_true` flags,
/// and interleaved positionals collected in order.
pub(crate) fn parse_flags(
    rest: &[String],
    value_specs: &[FlagSpec],
    boolean_specs: &[FlagSpec],
) -> Result<(Vec<String>, Vec<(String, Option<String>)>)> {
    let all: Vec<&FlagSpec> = value_specs.iter().chain(boolean_specs.iter()).collect();
    let mut positionals = Vec::new();
    let mut flags = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if !arg.starts_with("--") {
            positionals.push(arg);
            i += 1;
            continue;
        }
        let (name, inline_value) = match arg.split_once('=') {
            Some((n, v)) => (n.to_string(), Some(v.to_string())),
            None => (arg.clone(), None),
        };
        let spec = all
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("reference: unrecognized argument {name}"))?;
        if !spec.takes_value {
            flags.push((spec.name.to_string(), None));
            i += 1;
            continue;
        }
        let value = match inline_value {
            Some(v) => v,
            None => {
                i += 1;
                rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("reference: argument {name}: expected one argument")
                })?
            }
        };
        flags.push((spec.name.to_string(), Some(value)));
        i += 1;
    }
    Ok((positionals, flags))
}

pub(crate) fn require_flag(flags: &[(String, Option<String>)], name: &str) -> Result<String> {
    flags
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| v.clone())
        .ok_or_else(|| anyhow::anyhow!("reference: the following arguments are required: {name}"))
}

pub(crate) fn optional_flag(flags: &[(String, Option<String>)], name: &str) -> Option<String> {
    flags
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| v.clone())
}

pub(crate) fn require_positionals(positionals: &[String], names: &[&str]) -> Result<Vec<String>> {
    if positionals.len() < names.len() {
        bail!(
            "reference: the following arguments are required: {}",
            names[positionals.len()..].join(" ")
        );
    }
    Ok(positionals[..names.len()].to_vec())
}

pub(crate) const ADD_SPECS: &[FlagSpec] = &[
    FlagSpec {
        name: "--name",
        takes_value: true,
    },
    FlagSpec {
        name: "--source-url",
        takes_value: true,
    },
    FlagSpec {
        name: "--category",
        takes_value: true,
    },
    FlagSpec {
        name: "--selection-note",
        takes_value: true,
    },
    FlagSpec {
        name: "--visual",
        takes_value: true,
    },
    FlagSpec {
        name: "--owner",
        takes_value: true,
    },
];

/// `spis reference-record <add|get|remove> ...`
pub fn run(rest: &[String]) -> Result<()> {
    let Some(command) = rest.first() else {
        bail!(
            "reference: usage: spis reference-record <add|get|remove> [flags] \
             (add <catalog> --name N --source-url U --category C --selection-note S \
             --visual F [--owner O]; get <catalog> <NN|slug>; \
             remove <catalog> <NN|slug> [--force])"
        );
    };
    match command.as_str() {
        "add" => {
            let (mut positionals, flags) = parse_flags(&rest[1..], ADD_SPECS, &[])?;
            if positionals.is_empty() {
                bail!("reference: the following arguments are required: catalog");
            }
            let catalog = positionals.remove(0);
            add(&AddArgs {
                catalog,
                name: require_flag(&flags, "--name")?,
                source_url: require_flag(&flags, "--source-url")?,
                category: require_flag(&flags, "--category")?,
                selection_note: require_flag(&flags, "--selection-note")?,
                visual: require_flag(&flags, "--visual")?,
                owner: optional_flag(&flags, "--owner"),
            })
        }
        "get" | "remove" => {
            let extra_specs: &[FlagSpec] = if command == "get" {
                &[]
            } else {
                &[FlagSpec {
                    name: "--force",
                    takes_value: false,
                }]
            };
            let (positionals, flags) = parse_flags(&rest[1..], &[], extra_specs)?;
            let pos = require_positionals(&positionals, &["catalog", "identifier"])?;
            if command == "get" {
                get(&pos[0], &pos[1])
            } else {
                let force = flags.iter().any(|(n, _)| n == "--force");
                remove(&pos[0], &pos[1], force)
            }
        }
        other => bail!("reference: unknown command {other:?}"),
    }
}
