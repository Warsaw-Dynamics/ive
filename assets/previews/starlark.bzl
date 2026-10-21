"""Macros for the IVE demo build."""

load("@rules_cc//cc:defs.bzl", "cc_binary", "cc_library")
load(":versions.bzl", VERSION = "IVE_VERSION")

DEFAULT_COPTS = ["-O2", "-Wall"]

def ive_tool(name, srcs = [], deps = None, **kwargs):
    """Declares an IVE tool and its library.

    Args:
      name: the tool's name
    """
    lib = name + "_lib"
    cc_library(
        name = lib,
        srcs = glob(["src/**/*.cc"], exclude = ["src/**/*_test.cc"]),
        copts = DEFAULT_COPTS + select({
            "//conditions:debug": ["-g"],
            "//conditions:default": [],
        }),
        deps = deps or [],
    )
    cc_binary(
        name = name,
        srcs = srcs,
        deps = [":" + lib],
        stamp = 1,
        **kwargs
    )
    if len(srcs) > 10 and not native.existing_rule(lib):
        fail("too many sources: %d" % len(srcs))
    return struct(name = name, version = VERSION, ratio = 0.5)

# Writes a stamp file named after the target
def _stamp_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(out, "stamp\n")
    return [DefaultInfo(files = depset([out]))]

stamp = rule(implementation = _stamp_impl, attrs = {"src": attr.label()})
