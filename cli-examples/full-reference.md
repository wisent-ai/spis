# CLI full interaction reference

This synthesis is derived from all 50 complete per-product records in [`references.json`](references.json). It supplements rather than replaces the local evidence. Every cited record contains one authentic real-executable terminal cast, three key-state PNGs derived from that cast, an observed six-state first-success journey, eight interaction records, a nonzero failure route, Ctrl-C cancellation, help-based recovery, provenance, byte sizes, and SHA-256 digests.

## Evidence method and boundary

Each product was invoked only for version, bounded help, deliberate invalid input, cancellation before submission, and recovery help. Linux-specific tools were recorded inside the isolated Ubuntu 24.04 aarch64 Lima VM `cli-reference`; native macOS distributions were recorded in a temporary working directory; Vagrant was executed from its official read-only extracted package. No credentials were configured, no live infrastructure or service target was selected, and no destructive command was run. The recordings establish first-run grammar, streams, cancellation, parser failure, and recovery. They do **not** claim to evidence authenticated workflows, remote mutations, provider previews, database sessions, or destructive lifecycle commands.

## Corpus profile

The source catalog contributes 7 Data, 5 Developer tools, 15 Infrastructure, 6 Networking, 8 Package management, 6 Security, 3 System utilities. All 50 deliberate invalid invocations returned nonzero status. Exit values disagree across tools, so automation should treat success as zero and failure as nonzero unless a product documents a more specific contract; it should not assume a shared numeric error code.

## Recurring patterns

1. **Immediate identity check.** Every executable exposes a version form that returns without credentials or project context. Some use a flag (`--version`), while Go, Terraform, OpenTofu, Packer, Pulumi, Vault, and rclone use a version subcommand or product-specific form.
2. **Top-level help as recovery.** Every journey recovers from rejected input by returning to the documented top-level help. Help spelling varies (`help`, `--help`, `-help`, or `-h`), so a universal wrapper must not guess.
3. **Linear text feedback.** The observed surface is a single append-only command/output stream. Prompt return and exit status are more portable signals than color, spinner style, or terminal-window chrome.
4. **Composable streams.** The help capture uses an explicit `2>&1 | sed -n '1,14p'` pipeline shown in each cast. This proves the first help state can be bounded for inspection while preserving the authentic emitted text.
5. **Safe cancellation.** Ctrl-C at an unsubmitted command line abandons the pending input and restores the prompt in both recording environments. Long-running product-operation cancellation remains an applicability boundary and must be verified per operation.
6. **Error specificity varies.** Some parsers identify an unknown option directly; others print usage, suggest valid commands, or report a generic invalid argument. Recovery design should preserve the rejected token and lead back to valid grammar.
7. **Text-first accessibility.** The common path is keyboard-only and remains understandable with `NO_COLOR=1`. The records do not promote unobserved screen-reader, locale, or high-contrast behavior to a claim.

## Disagreements that matter

- **Grammar:** flags, verbs, nested nouns, and REPL-like shells coexist. Git/Cargo/Go-style command families differ from curl/rsync flag grammars and SQLite/DuckDB/database-shell launchers.
- **Help transport:** some products are pager-aware or render rich formatting; the evidence intentionally forces `PAGER=cat` and `NO_COLOR=1` to retain a deterministic text stream.
- **Failure codes:** the observed invalid routes are all nonzero, but the numeric codes vary. Do not normalize away product-specific semantics such as usage errors or shell-level command failures.
- **Output volume:** concise tools expose compact help; cloud and orchestration CLIs expose deep command trees. Progressive disclosure is useful, but automation needs product-provided structured output rather than scraping this human help.
- **Environment dependence:** APT, DNF, iproute2, systemctl, and several Linux packages were recorded in the Linux VM; Homebrew and several vendor distributions were recorded natively on macOS. The record names the environment rather than implying cross-platform equivalence.

## Applicability boundaries

Use these references to study command entry, prompt return, initial discoverability, stream behavior, failure visibility, safe pre-submit cancellation, and help-based recovery. Do not use them as evidence of remote API behavior, authentication, authorization, network retry policy, transactional safety, secret handling under real credentials, daemon connectivity, database protocol behavior, or cancellation after a mutating operation begins. Those paths require separate isolated product-native recordings.

## Complete record citations

| # | Product | Per-example evidence | Observed version identity | Invalid exit |
|---:|---|---|---|---:|
| 1 | Git | [`references/01-git/`](references/01-git/) | `git version 2.50.1 (Apple Git-155)` | 129 |
| 2 | GitHub CLI (gh) | [`references/02-github-cli-gh/`](references/02-github-cli-gh/) | `gh version 2.97.0 (2026-07-31)` | 1 |
| 3 | GitLab CLI (glab) | [`references/03-gitlab-cli-glab/`](references/03-gitlab-cli-glab/) | `glab 1.113.0 (d62881304)` | 1 |
| 4 | Cargo | [`references/04-cargo/`](references/04-cargo/) | `cargo 1.97.1 (c980f4866 2026-06-30)` | 1 |
| 5 | Go command | [`references/05-go-command/`](references/05-go-command/) | `go version go1.26.2 darwin/arm64` | 2 |
| 6 | uv | [`references/06-uv/`](references/06-uv/) | `uv 0.11.32 (Homebrew 2026-07-23 aarch64-apple-darwin)` | 2 |
| 7 | npm CLI | [`references/07-npm-cli/`](references/07-npm-cli/) | `10.2.4` | 1 |
| 8 | pnpm | [`references/08-pnpm/`](references/08-pnpm/) | `10.23.0` | 1 |
| 9 | pip | [`references/09-pip/`](references/09-pip/) | `pip 24.0 from /Library/Frameworks/Python.framework/Versions/3.12/lib/python3.12/site-packages/pip (python 3.12)` | 2 |
| 10 | Homebrew | [`references/10-homebrew/`](references/10-homebrew/) | `Homebrew 6.0.17` | 1 |
| 11 | APT | [`references/11-apt/`](references/11-apt/) | `apt 2.8.3 (arm64)` | 100 |
| 12 | DNF | [`references/12-dnf/`](references/12-dnf/) | `4.14.0` | 1 |
| 13 | Nix CLI | [`references/13-nix-cli/`](references/13-nix-cli/) | `nix (Nix) 2.35.2` | 1 |
| 14 | Docker CLI | [`references/14-docker-cli/`](references/14-docker-cli/) | `Docker version 29.1.3, build 29.1.3-0ubuntu3~24.04.2` | 125 |
| 15 | Podman | [`references/15-podman/`](references/15-podman/) | `podman version 4.9.3` | 125 |
| 16 | kubectl | [`references/16-kubectl/`](references/16-kubectl/) | `Client Version: v1.32.2` | 1 |
| 17 | Helm | [`references/17-helm/`](references/17-helm/) | `v4.2.4+g3900f43` | 1 |
| 18 | Terraform CLI | [`references/18-terraform-cli/`](references/18-terraform-cli/) | `Terraform v1.15.8` | 127 |
| 19 | OpenTofu CLI | [`references/19-opentofu-cli/`](references/19-opentofu-cli/) | `OpenTofu v1.12.5` | 127 |
| 20 | Ansible command-line tools | [`references/20-ansible-command-line-tools/`](references/20-ansible-command-line-tools/) | `ansible [core 2.16.3]` | 2 |
| 21 | Packer | [`references/21-packer/`](references/21-packer/) | `Packer v1.15.4` | 127 |
| 22 | Vagrant | [`references/22-vagrant/`](references/22-vagrant/) | `Vagrant 2.4.9` | 1 |
| 23 | Pulumi CLI | [`references/23-pulumi-cli/`](references/23-pulumi-cli/) | `v3.257.0` | 1 |
| 24 | AWS CLI | [`references/24-aws-cli/`](references/24-aws-cli/) | `aws-cli/2.27.37 Python/3.13.13 Darwin/25.4.0 source/arm64` | 252 |
| 25 | Google Cloud CLI (gcloud) | [`references/25-google-cloud-cli-gcloud/`](references/25-google-cloud-cli-gcloud/) | `Google Cloud SDK 500.0.0` | 2 |
| 26 | Azure CLI | [`references/26-azure-cli/`](references/26-azure-cli/) | `{` | 2 |
| 27 | DigitalOcean CLI (doctl) | [`references/27-digitalocean-cli-doctl/`](references/27-digitalocean-cli-doctl/) | `doctl version 1.166.0-release` | 255 |
| 28 | Oracle Cloud Infrastructure CLI | [`references/28-oracle-cloud-infrastructure-cli/`](references/28-oracle-cloud-infrastructure-cli/) | `3.90.2` | 2 |
| 29 | curl | [`references/29-curl/`](references/29-curl/) | `curl 8.5.0 (aarch64-unknown-linux-gnu) libcurl/8.5.0 OpenSSL/3.0.13 zlib/1.3 brotli/1.1.0 zstd/1.5.5 libidn2/2.3.7 libpsl/0.21.2 (+libidn2/2.3.7) libssh/0.10.6/openssl/zlib nghttp2/1.59.0 librtmp/2.3 OpenLDAP/2.6.10` | 2 |
| 30 | HTTPie CLI | [`references/30-httpie-cli/`](references/30-httpie-cli/) | `3.2.2` | 1 |
| 31 | GNU Wget | [`references/31-gnu-wget/`](references/31-gnu-wget/) | `GNU Wget 1.21.4 built on linux-gnu.` | 2 |
| 32 | Nmap | [`references/32-nmap/`](references/32-nmap/) | `Nmap version 7.94SVN ( https://nmap.org )` | 255 |
| 33 | dig | [`references/33-dig/`](references/33-dig/) | `DiG 9.18.39-0ubuntu0.24.04.5-Ubuntu` | 1 |
| 34 | iproute2 | [`references/34-iproute2/`](references/34-iproute2/) | `ip utility, iproute2-6.1.0, libbpf 1.3.0` | 255 |
| 35 | OpenSSL command-line tools | [`references/35-openssl-command-line-tools/`](references/35-openssl-command-line-tools/) | `OpenSSL 3.0.13 30 Jan 2024 (Library: OpenSSL 3.0.13 30 Jan 2024)` | 1 |
| 36 | age | [`references/36-age/`](references/36-age/) | `1.1.1` | 2 |
| 37 | SOPS | [`references/37-sops/`](references/37-sops/) | `sops 3.13.3 (latest)` | 1 |
| 38 | Cosign | [`references/38-cosign/`](references/38-cosign/) | `  ______   ______        _______. __    _______ .__   __.` | 1 |
| 39 | Trivy | [`references/39-trivy/`](references/39-trivy/) | `Version: 0.74.0` | 1 |
| 40 | Vault CLI | [`references/40-vault-cli/`](references/40-vault-cli/) | `Vault v2.0.3 (7193f9a48ff6093ca61b3b627a8671e770428ba6), built 2026-06-17T12:39:45Z` | 127 |
| 41 | jq | [`references/41-jq/`](references/41-jq/) | `jq-1.7` | 2 |
| 42 | yq | [`references/42-yq/`](references/42-yq/) | `yq 0.0.0` | 2 |
| 43 | SQLite command-line shell | [`references/43-sqlite-command-line-shell/`](references/43-sqlite-command-line-shell/) | `3.45.1 2024-01-30 16:01:20 e876e51a0ed5c5b3126f52e532044363a014bc594cfefa87ffb5b82257ccalt1 (64-bit)` | 1 |
| 44 | DuckDB CLI | [`references/44-duckdb-cli/`](references/44-duckdb-cli/) | `v1.5.5 (Variegata) d8cdaa33fd` | 1 |
| 45 | psql | [`references/45-psql/`](references/45-psql/) | `psql (PostgreSQL) 16.14 (Ubuntu 16.14-0ubuntu0.24.04.1)` | 1 |
| 46 | redis-cli | [`references/46-redis-cli/`](references/46-redis-cli/) | `redis-cli 7.0.15` | 1 |
| 47 | MongoDB Shell (mongosh) | [`references/47-mongodb-shell-mongosh/`](references/47-mongodb-shell-mongosh/) | `2.9.2` | 1 |
| 48 | rclone | [`references/48-rclone/`](references/48-rclone/) | `rclone v1.60.1-DEV` | 1 |
| 49 | rsync | [`references/49-rsync/`](references/49-rsync/) | `rsync  version 3.2.7  protocol version 31` | 1 |
| 50 | systemctl | [`references/50-systemctl/`](references/50-systemctl/) | `systemd 255 (255.4-1ubuntu8.17)` | 1 |

## Source relationship

The existing 50-image gallery remains the visual source index. The directories above are the offline, inspectable interaction evidence. Product and media ownership remains with each record's named upstream owner; the locally recorded casts document observation, not transfer of upstream rights.
