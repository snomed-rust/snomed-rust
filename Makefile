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
# as the simplest way to reproduce that push by hand. Delegates to
# bin/make-github-pages (a standalone POSIX script, not inlined here) —
# see that script's header for the `github-pages` remote it requires.
github-pages:
	bin/make-github-pages
