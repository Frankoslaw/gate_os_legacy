# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

define DEFAULT_VAR =
    ifeq ($(origin $1),default)
        override $(1) := $(2)
    endif
    ifeq ($(origin $1),undefined)
        override $(1) := $(2)
    endif
endef


# Target architecture to build for. Default to x86_64.
override DEFAULT_ARCH := x86_64
override DEFAULT_RUST_TARGET := x86_64-unknown-none
$(eval $(call DEFAULT_VAR,ARCH,$(DEFAULT_ARCH)))


ifeq ($(ARCH),x86_64)
    override DEFAULT_RUST_TARGET := x86_64-unknown-none
endif

ifeq ($(ARCH),aarch64)
    override DEFAULT_RUST_TARGET := aarch64-unknown-none
endif

ifeq ($(ARCH),riscv64)
    override DEFAULT_RUST_TARGET := riscv64gc-unknown-none-elf
endif

$(eval $(call DEFAULT_VAR,RUST_TARGET,$(DEFAULT_RUST_TARGET)))

override DEFAULT_RUST_PROFILE := dev
$(eval $(call DEFAULT_VAR,RUST_PROFILE,$(DEFAULT_RUST_PROFILE)))

override RUST_PROFILE_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
    override RUST_PROFILE_SUBDIR := debug
endif

# Default target.
.PHONY: all
all:
	cargo rustc --target $(RUST_TARGET) --profile $(RUST_PROFILE) -- -C link-arg=-Tlinker-$(ARCH).ld
	cp target/$(RUST_TARGET)/$(RUST_PROFILE_SUBDIR)/$$(cd target/$(RUST_TARGET)/$(RUST_PROFILE_SUBDIR) && find -maxdepth 1 -perm -111 -type f) kernel

# Remove object files and the final executable.
.PHONY: clean
clean:
	cargo clean
	rm -rf kernel

.PHONY: distclean
distclean: clean