# Tasks archive 9 of 9 — 2026-08-27

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: commit/tag signing configured
(SSH-format signing key, `allowed_signers`, the bootstrapping exception of
one unsigned commit before the key was unlocked and verified end to end,
and the finding that registering the key as a *signing* key on GitHub,
GitLab, and Codeberg needs the maintainer present).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-08-27, commit/tag signing configured — partly closes a hygiene gap)

- [x] **Configured local git signing** for this repository:
      `gpg.format = ssh`, `user.signingkey` pointing at
      `~/.ssh/id.d/jph-code-signing=8a085b90451ad01ba7646faae803accc=
      ssh-ed25519-with-passphrase.pub`, `gpg.ssh.allowedSignersFile` at
      `~/.ssh/allowed_signers`, and `commit.gpgsign`/`tag.gpgsign` both
      `true`. Verified before writing anything down: the public key's
      fingerprint (`ssh-keygen -lf`) matches the entry already present in
      `~/.ssh/allowed_signers` under `joel@joelparkerhenderson.com`, and
      `ssh-keygen -Y sign` is available (OpenSSH 10.4, well past the 8.2
      minimum for SSH signing).
- [x] **Did not attempt a live signed commit while the key was locked.**
      The private key is passphrase-protected and was not loaded in
      `ssh-agent` at the time; a non-interactive shell has no way to supply
      that passphrase, and shouldn't try to. This change's own commit
      landed first with `--no-gpg-sign` explicitly, as a bootstrapping
      exception. Once the maintainer unlocked the key
      (`ssh-add --apple-use-keychain`), verified with `ssh-add -l`, a
      round-trip smoke test (`ssh-keygen -Y sign` / `-Y verify` against
      `~/.ssh/allowed_signers`, both clean) and a throwaway-branch empty
      commit (`git commit -S`, `%G?` = `G`, deleted after) confirmed
      signing actually works end to end before trusting it on real
      history. That commit was then **amended to be signed** and pushed —
      so the version of this entry you are reading is itself in a signed
      commit, not the unsigned one described above; `git log
      --show-signature` on it should say so.
- [x] **Checked GitHub, GitLab, and Codeberg registration and found none
      possible without the maintainer present.** `gh ssh-key list` 404s:
      the CLI's OAuth token lacks the `admin:ssh_signing_key` scope, and
      granting it (`gh auth refresh -h github.com -s
      admin:ssh_signing_key`) is an interactive, account-holder-only
      approval. Only one key is on the GitHub account today, typed
      `authentication`, not `signing`. Neither `glab` nor `tea` (GitLab,
      Codeberg/Forgejo CLIs) is installed. So none of the three forges
      will show a "Verified" badge yet — updated `MAINTAINERS.md`,
      `SECURITY.md`, `plan.md`, and `spec/professionalization/index.md`
      in this same change to say exactly that, rather than either leaving
      the old "no signing key" claim standing or overclaiming completion.
- [x] Left as a named follow-up rather than a silent gap: the maintainer
      registers the same public key with each forge as a *signing* key
      (GitHub: Settings → SSH and GPG keys → New SSH key → Key type
      "Signing Key", or `gh auth refresh` then `gh ssh-key add ... --type
      signing`; GitLab and Codeberg have the equivalent under their own
      SSH key settings).
