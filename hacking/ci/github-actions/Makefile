root_dir := ../../..

.PHONY: none
none:

.PHONY: check
check:
	$(MAKE) -C $(root_dir) check-oneshot

	# For debugging and monitoring disk usage
	# ($(MAKE) -C $(root_dir) check-oneshot && df -h) || (df -h && false)

.PHONY: docs
docs:
	$(MAKE) -C $(root_dir) html
