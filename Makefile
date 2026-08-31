# Publishing the website.
#
# snomed-rust.github.io/ is a git subtree, not a submodule or a separate
# checkout. `git subtree split` replays that directory's history as a
# standalone commit whose root is the site itself, which is the shape the
# pages repo needs; pushing it there fires that repo's deploy workflow.
#
# Publishing this way uses the pusher's own credentials, so there is no
# deploy key, no GitHub App secret, and no org-wide setting behind it.

PAGES_REMOTE := pages
GITHUB_PAGES_REMOTE := github-pages
PAGES_PREFIX := snomed-rust.github.io

.PHONY: publish github-pages

publish:
	@git diff --quiet --ignore-submodules HEAD -- $(PAGES_PREFIX) \
		|| { echo "error: uncommitted changes in $(PAGES_PREFIX); commit them first"; exit 1; }
	git fetch $(PAGES_REMOTE)
	git push $(PAGES_REMOTE) \
		"$$(git subtree split -q --prefix=$(PAGES_PREFIX))":refs/heads/main \
		--force-with-lease

# The plain `git subtree push` porcelain: same subtree-split-and-push as
# `publish` above, but without its dirty-tree guard or `--force-with-lease`
# (a `git subtree push` refuses non-fast-forwards outright rather than
# safely forcing past them). Prefer `publish` day to day; this target is
# the direct command per spec/monorepo-github-pages/index.md, kept around
# as the simplest way to reproduce that push by hand. Uses its own remote
# name, `github-pages` (`git remote add github-pages
# git@github.com:snomed-rust/snomed-rust.github.io.git` — today the same
# URL as `pages` above), so this target matches that spec verbatim.
github-pages:
	git subtree push --prefix=$(PAGES_PREFIX) $(GITHUB_PAGES_REMOTE) main
