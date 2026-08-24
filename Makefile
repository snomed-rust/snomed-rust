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

.PHONY: publish

publish:
	@git diff --quiet --ignore-submodules HEAD -- $(PAGES_PREFIX) \
		|| { echo "error: uncommitted changes in $(PAGES_PREFIX); commit them first"; exit 1; }
	git fetch $(PAGES_REMOTE)
	git push $(PAGES_REMOTE) \
		"$$(git subtree split -q --prefix=$(PAGES_PREFIX))":refs/heads/main \
		--force-with-lease
