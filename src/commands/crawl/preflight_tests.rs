use super::*;

#[test]
fn every_family_declares_the_worker_program_it_actually_runs() {
    // The defect this closes: the documentation engine was the only one
    // probing cargo, while all six submit `cargo run --release`.
    for (catalog, engine) in CATALOGS {
        let required = engine_preconditions(engine, catalog);
        assert!(
            required.contains(&WORKER_PROGRAM.to_vec()),
            "{catalog} ({engine}) does not declare the worker program"
        );
    }
    assert_eq!(CATALOGS.len(), 15, "the family count is part of this claim");
}

#[test]
fn each_engine_declares_the_runtime_its_worker_drives() {
    let required = |catalog: &str| {
        let engine = CATALOGS
            .iter()
            .find(|(name, _)| *name == catalog)
            .map(|(_, engine)| *engine)
            .expect("a known catalog");
        engine_preconditions(engine, catalog)
            .into_iter()
            .map(|command| command.join(" "))
            .collect::<Vec<String>>()
    };
    let ios = required("ios-app-examples");
    assert!(ios.iter().any(|command| command.starts_with("appium --version")));
    assert!(ios.iter().any(|command| command.contains("simctl")));
    // iOS drives a simulator, so it must NOT demand the Android bridge.
    assert!(!ios.iter().any(|command| command.starts_with("adb ")));
    let android = required("android-app-examples");
    assert!(android.iter().any(|command| command == "adb version"));
    assert!(android.iter().any(|command| command == "adb devices -l"));
    assert!(!android.iter().any(|command| command.contains("simctl")));
    for terminal in ["cli-examples", "tui-examples"] {
        assert!(required(terminal).iter().any(|command| command == "tmux -V"));
    }
    for web in [
        "web-app-examples",
        "dashboard-console-examples",
        "onboarding-auth-examples",
        "app-store-listing-examples",
        "design-system-examples",
        "report-evidence-examples",
        "pricing-page-examples",
        "landing-page-examples",
    ] {
        assert!(required(web).iter().any(|command| command == "node --version"));
    }
    // Desktop's driver is checked by absolute bundle candidate inside
    // `host_preflight`, so its declared set is the worker program alone.
    assert_eq!(required("macos-app-examples"), vec!["cargo --version".to_string()]);
}

#[test]
fn the_terminal_family_declares_the_git_its_worker_builds_a_fixture_with() {
    let tui = engine_preconditions("tui", "tui-examples")
        .into_iter()
        .map(|command| command.join(" "))
        .collect::<Vec<String>>();
    assert!(tui.iter().any(|command| command == "git --version"));
    assert!(tui.iter().any(|command| command == "tmux -V"));
    // The CLI worker builds no repository, so it must not demand git.
    let cli = engine_preconditions("cli", "cli-examples")
        .into_iter()
        .map(|command| command.join(" "))
        .collect::<Vec<String>>();
    assert!(!cli.iter().any(|command| command.starts_with("git ")));
}

#[test]
fn no_declared_precondition_names_the_xcode_select_shim() {
    // The probe must never be the `/usr/bin/git` shim: on a host without
    // the Command Line Tools it opens the installer WINDOW. The allowlist
    // excludes that path, and nothing here may ask for it directly.
    for (catalog, engine) in CATALOGS {
        for command in engine_preconditions(engine, catalog) {
            assert_ne!(command[0], "/usr/bin/git", "{catalog}");
        }
    }
}

#[test]
fn every_declared_precondition_names_a_program_and_never_a_bare_search() {
    // A probe must be an exact approved entry, never something a shell
    // would have to resolve through PATH at job time.
    for (catalog, engine) in CATALOGS {
        for command in engine_preconditions(engine, catalog) {
            assert!(!command.is_empty(), "{catalog}: empty precondition");
            assert!(
                !command[0].contains('/') || command[0].starts_with('/'),
                "{catalog}: {} is neither a program name nor an absolute path",
                command[0]
            );
            assert!(
                !command.iter().any(|word| word.contains('~')),
                "{catalog}: {} carries a tilde no shell will expand here",
                command.join(" ")
            );
        }
    }
}

#[test]
fn a_receipt_pin_is_read_in_the_consumer_spelling_stado_answers_with() {
    // Measured on run `docs-50-222852`: the pin was submitted as
    // `charless-mac-mini` and echoed as `local-charless-mac-mini.local`,
    // and the literal comparison recorded thirty-seven accepted
    // submissions as failures while their jobs ran.
    assert!(super::pinned_host_is("charless-mac-mini", "charless-mac-mini"));
    assert!(super::pinned_host_is(
        "local-charless-mac-mini.local",
        "charless-mac-mini"
    ));
    assert!(super::pinned_host_is(
        "local-Charless-Mac-mini.local",
        "charless-mac-mini"
    ));
    // A different host, and the two shapes that only look like the
    // consumer spelling, are still refused.
    assert!(!super::pinned_host_is(
        "local-lukasz-macbook.local",
        "charless-mac-mini"
    ));
    assert!(!super::pinned_host_is("charless-mac-mini.local", "charless-mac-mini"));
    assert!(!super::pinned_host_is("local-charless-mac-mini", "charless-mac-mini"));
}
