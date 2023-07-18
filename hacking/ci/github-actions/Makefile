root_dir := ../../..

.PHONY: none
none:

.PHONY: check
check:
	($(MAKE) -C $(root_dir) check-oneshot && df -h) || (df -h && false)
