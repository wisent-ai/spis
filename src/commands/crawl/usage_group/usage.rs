use super::*;

pub(crate) fn usage() {
    println!(
        "usage:
  spis crawl bindings generate --weles-token-ref ITEM#FIELD --organization-ref ITEM#FIELD [--output PATH]
  spis crawl start [--host ENGINE=TARGET] [--catalog SLUG ...] [--record SLUG] [--run-id ID] [--bindings PATH]
  spis crawl status [--run RUN_ID] [--record SLUG]
  spis crawl cancel --run RUN_ID [--record SLUG] --reason TEXT
  spis crawl resume --run RUN_ID [--record SLUG]
  spis crawl import --run RUN_ID [--record SLUG]

Every command emits exactly one JSON document on stdout; the CLI is the process
API and Spis exposes no second HTTP crawl surface.

Each record is one immutable attempt: identity, both stado:// attempt URIs and
the Stado run id are derived from the record key, and every state transition is
persisted under a durable per-record lock before the external effect it authorizes.

  bindings generate  Writes the exact typed binding for every checked-in record.
                     With --output an existing generated document is replaced
                     atomically after validation and read-back; the reported
                     outcome is created, replaced or unchanged.
  start              Idempotent. Re-running the same request digest continues the
                     existing run; planned, preflight_passed and submitting
                     records are driven forward and a record held by another
                     process is skipped, never failed.
  status             Refreshes from Stado when the run lock is free, otherwise
                     returns a read-only snapshot.
  cancel             Status-first, durable and idempotent. The intent is recorded
                     locally and published immutably before any cancellation is
                     dispatched, so a crash can never resurrect the record.
  resume             Never reruns a Stado job. A terminal failed, cancelled, lost
                     or submission_failed attempt becomes attempt N+1 with fully
                     recomputed identity; queued and running records are left
                     alone; completed records are imported.
  import             Per record and per attempt. The typed worker report, the
                     Stado submission receipt, the attempt artifact digest and
                     byte count and every retained evidence hash are verified
                     before a staged, fsynced, atomically installed record
                     transaction; earlier attempts and their diagnostics are kept."
    );
}

/// `spis crawl preflight --catalog C --host H [--json]` — can this host run
/// this family's worker, asked without claiming anything.
///
/// Read-only: it runs the family's declared preconditions through the
/// approved host-exec entries and prints the verdict. It touches no run
/// store, submits no job and takes no slot, which is the whole point --
/// before this the only way to find out was to drive a crawl, and the
/// documentation engine proved what that costs when the answer is no:
/// job-545551889f9e88be30daa81f held a slot for sixteen minutes to discover
/// a missing program.
///
/// Exits non-zero when the host is not ready, so a placement script can ask
/// this question in a condition.
pub(crate) fn preflight(rest: &[String]) -> Result<()> {
    let mut catalog = None;
    let mut host = None;
    let mut json_output = false;
    let mut index = 0usize;
    while index < rest.len() {
        match rest[index].as_str() {
            "--catalog" => {
                index += 1;
                catalog = Some(rest.get(index).context("--catalog needs a value")?.clone());
            }
            "--host" => {
                index += 1;
                host = Some(rest.get(index).context("--host needs a value")?.clone());
            }
            "--json" => json_output = true,
            other => bail!("unknown argument: {other}"),
        }
        index += 1;
    }
    let catalog = catalog.context("preflight needs --catalog")?;
    let host = host.context("preflight needs --host")?;
    safe_component(&catalog, "--catalog")?;
    safe_component(&host, "--host")?;
    let engine = CATALOGS
        .iter()
        .find(|(name, _)| *name == catalog.as_str())
        .map(|(_, engine)| *engine)
        .with_context(|| format!("{catalog} is not a Spis catalog"))?;
    // No service identity: the Weles release check belongs to a run that has
    // resolved one, and inventing it here would report a readiness this
    // command cannot stand behind. The web engine therefore reports its
    // program preconditions and says the admission check is unresolved.
    let report = host_preflight(&catalog, engine, &host, None);
    let ready = report.get("ready").and_then(Value::as_bool) == Some(true);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("catalog:  {catalog}");
        println!("engine:   {engine}");
        println!("host:     {host}");
        println!("ready:    {ready}");
        for check in report
            .get("checks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let command = check
                .get("command")
                .and_then(Value::as_array)
                .map(|words| {
                    words
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<&str>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let verdict = if check.get("ready").and_then(Value::as_bool) == Some(true) {
                "ok"
            } else {
                "MISSING"
            };
            let detail = check
                .get("error")
                .and_then(Value::as_str)
                .or_else(|| check.get("stdout").and_then(Value::as_str))
                .unwrap_or_default()
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            println!("  {verdict:<8} {command:<52} {detail}");
        }
    }
    if !ready {
        bail!("host {host} cannot run the {catalog} worker; every MISSING line above is a precondition this family declares");
    }
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    match rest.first().map(String::as_str) {
        Some("start") => start(&rest[1..]),
        Some("preflight") => preflight(&rest[1..]),
        Some("status") => status(&rest[1..]),
        Some("cancel") => cancel(&rest[1..]),
        Some("resume") => resume(&rest[1..]),
        Some("import") => import(&rest[1..]),
        Some("--help" | "-h") | None => { usage(); Ok(()) }
        Some("bindings") if rest.get(1).map(String::as_str) == Some("generate") => {
            generate_runtime_bindings(&rest[2..])
        }
        Some(other) => bail!("unknown crawl operation: {other}"),
    }
}
