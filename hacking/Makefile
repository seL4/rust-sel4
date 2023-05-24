top_level_dir = ..
hacking_dir = .

.PHONY: none
none:

.PHONY: check
check:
	$(MAKE) -C $(top_level_dir) check-generated-sources
	$(MAKE) -C $(top_level_dir) fmt-check
	$(MAKE) -C $(hacking_dir) check-generic-formatting
	$(MAKE) -C $(top_level_dir) run-automated-tests
	$(MAKE) -C $(top_level_dir) everything-with-excess

.PHONY: check-generic-formatting
check-generic-formatting:
	$(hacking_dir)/scripts/check-generic-formatting.sh
