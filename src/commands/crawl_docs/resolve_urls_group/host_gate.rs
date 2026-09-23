use super::*;

// ---------- parallel fetch engine ----------

pub(crate) struct HostGate {
    pub(crate) next_allowed: Mutex<HashMap<String, std::time::Instant>>,
    pub(crate) host_delay: f64,
}

impl HostGate {
    pub(crate) fn new(host_delay: f64) -> Self {
        Self {
            next_allowed: Mutex::new(HashMap::new()),
            host_delay,
        }
    }

    /// Block until this host's next slot, then reserve it.
    pub(crate) fn wait_turn(&self, url: &str) {
        loop {
            let now = std::time::Instant::now();
            let host = lib::origin_of(url);
            let mut slots = self.next_allowed.lock();
            let slot = slots.entry(host).or_insert(now);
            if *slot <= now {
                *slot = now + std::time::Duration::from_secs_f64(self.host_delay);
                return;
            }
            let wait = *slot - now;
            drop(slots);
            std::thread::sleep(wait);
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct CrawlTarget {
    pub(crate) sequence: usize,
    pub(crate) key: String,
    pub(crate) url: String,
    pub(crate) lastmod: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct CrawlDiagnostic {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) url: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct PageOutcome {
    pub(crate) sequence: usize,
    pub(crate) url: String,
    pub(crate) resolved_url: String,
    pub(crate) status: Value,
    pub(crate) diagnostic: Option<CrawlDiagnostic>,
    pub(crate) text_bytes: Option<u64>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) record_sha256: Option<String>,
    pub(crate) corpus_start: Option<u64>,
    pub(crate) corpus_end: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct JournalOutcome {
    pub(crate) key: String,
    pub(crate) outcome: PageOutcome,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct OutcomeJournalBatch {
    pub(crate) schema: String,
    pub(crate) first_sequence: usize,
    pub(crate) last_sequence: usize,
    pub(crate) committed_bytes: u64,
    pub(crate) committed_sha256: String,
    pub(crate) outcomes: Vec<JournalOutcome>,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct DurableState {
    pub(crate) schema: String,
    pub(crate) run_id: String,
    pub(crate) source_revision: String,
    pub(crate) source_input_sha256: String,
    pub(crate) record_key: String,
    pub(crate) record: String,
    pub(crate) attempt: u32,
    pub(crate) attempt_id: String,
    pub(crate) source_url: String,
    pub(crate) started_at: String,
    pub(crate) effective_source_url: String,
    pub(crate) inventory_complete: bool,
    pub(crate) inventory_downloaded_bytes: u64,
    pub(crate) inventory_sha256: Option<String>,
    pub(crate) inventory_diagnostics: Vec<CrawlDiagnostic>,
    /// What the inventory could not fit, absent for every record that fits.
    ///
    /// Optional and `skip_serializing_if` so a corpus written before this
    /// existed serialises byte-identically and keeps its inventory digest;
    /// present, it is folded into that digest, because a count of what a
    /// record is missing is part of what the record's inventory says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) corpus_capacity: Option<CorpusCapacity>,
    pub(crate) robots: Option<RobotsSnapshot>,
    pub(crate) targets: Vec<CrawlTarget>,
    pub(crate) outcomes: BTreeMap<String, PageOutcome>,
    pub(crate) committed_bytes: u64,
    pub(crate) committed_sha256: String,
    pub(crate) completed_at: Option<String>,
    pub(crate) report_sha256: Option<String>,
}

pub(crate) struct FetchedOutcome {
    pub(crate) target: CrawlTarget,
    pub(crate) status: Value,
    pub(crate) diagnostic: Option<CrawlDiagnostic>,
    pub(crate) text_bytes: Option<u64>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) line: Option<Vec<u8>>,
    pub(crate) resolved_url: String,
}

pub(crate) struct WriteRequest {
    pub(crate) outcome: FetchedOutcome,
    pub(crate) acknowledge: mpsc::Sender<std::result::Result<(), String>>,
}

pub(crate) enum WriterMessage {
    Outcome(WriteRequest),
    Abort(String),
}

pub(crate) struct FetchShared {
    pub(crate) queue: Mutex<std::vec::IntoIter<CrawlTarget>>,
    pub(crate) writer: mpsc::Sender<WriterMessage>,
    pub(crate) gate: HostGate,
    pub(crate) policy: UrlPolicy,
    pub(crate) downloaded_bytes: AtomicU64,
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) robots: CompiledRobots,
}

#[derive(Clone)]
pub(crate) struct WorkLayout {
    pub(crate) root: PathBuf,
    pub(crate) corpus: PathBuf,
    pub(crate) state: PathBuf,
    pub(crate) pages: PathBuf,
    pub(crate) journal: PathBuf,
    pub(crate) report: PathBuf,
}

pub(crate) struct WorkLock {
    pub(crate) file: File,
}

impl WorkLock {
    pub(crate) fn acquire(layout: &WorkLayout) -> Result<Self> {
        std::fs::create_dir_all(&layout.root)
            .with_context(|| format!("create durable work directory {}", layout.root.display()))?;
        let parent = layout
            .root
            .parent()
            .context("durable work directory has no parent")?;
        let name = layout
            .root
            .file_name()
            .and_then(|value| value.to_str())
            .context("durable work directory has no UTF-8 name")?;
        let file = open_regular_file(
            &parent.join(format!(".{name}.crawl.lock")),
            true,
            true,
            false,
            true,
            "durable work lock",
        )?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!(
                "documentation crawl {} is already active in another worker",
                layout.root.display()
            );
        }
        Ok(Self { file })
    }
}

impl Drop for WorkLock {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

#[derive(Clone)]
pub(crate) struct WorkerOptions {
    pub(crate) site: Option<String>,
    pub(crate) all: bool,
    pub(crate) exclude: Vec<String>,
    pub(crate) workers: usize,
    pub(crate) host_delay: f64,
    pub(crate) refresh: bool,
}

impl WorkerOptions {
    pub(crate) fn parse(rest: &[String]) -> Result<Self> {
        let mut options = Self {
            site: None,
            all: false,
            exclude: Vec::new(),
            workers: MAX_WORKERS,
            host_delay: 0.3,
            refresh: false,
        };
        let mut i = 0;
        while i < rest.len() {
            match rest[i].as_str() {
                "--site" => {
                    i += 1;
                    options.site = Some(rest.get(i).context("--site needs a value")?.clone());
                }
                "--all" => options.all = true,
                "--exclude" => {
                    i += 1;
                    options
                        .exclude
                        .push(rest.get(i).context("--exclude needs a value")?.clone());
                }
                "--workers" => {
                    i += 1;
                    options.workers = rest.get(i).context("--workers needs a value")?.parse()?;
                }
                "--host-delay" => {
                    i += 1;
                    options.host_delay =
                        rest.get(i).context("--host-delay needs a value")?.parse()?;
                }
                "--refresh" => options.refresh = true,
                other => bail!("unknown argument: {other}"),
            }
            i += 1;
        }
        if options.site.is_none() && !options.all {
            bail!("pass --site <NN-slug> or --all");
        }
        if options.workers == 0 || options.workers > MAX_WORKERS {
            bail!("--workers must be between 1 and {MAX_WORKERS}");
        }
        if !options.host_delay.is_finite()
            || !(0.0..=MAX_HOST_DELAY_SECONDS).contains(&options.host_delay)
        {
            bail!(
                "--host-delay must be a finite number between 0 and {MAX_HOST_DELAY_SECONDS} seconds"
            );
        }
        Ok(options)
    }
}

pub(crate) fn safe_path_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{label} contains unsafe durable-path characters");
    }
    Ok(())
}
