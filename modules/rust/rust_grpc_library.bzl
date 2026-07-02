"""Generated definition of rust_grpc_library."""

load("@rules_proto_grpc//:defs.bzl", "bazel_build_rule_common_attrs", "proto_compile_attrs")
load("@rules_rust//rust:defs.bzl", "rust_library")
load(":common.bzl", "crate_label", "merge_rust_deps", "rust_compile_attrs", "rust_proto_library_forward")
load(":rust_fixer.bzl", "rust_proto_crate_fixer", "rust_proto_crate_root")
load(":rust_grpc_compile.bzl", "rust_grpc_compile")

def rust_grpc_library(name, **kwargs):
    """Generates Rust grpc code and wraps it in a rust_library.

    Args:
        name: Name of the generated rust_library target.
        **kwargs: Common Bazel attributes are forwarded to both generated
            targets; proto compile attributes are forwarded to
            rust_grpc_compile; Rust-specific attributes
            such as crate_name and declared_proto_packages configure crate
            generation. proto_deps is deprecated; use deps for Rust proto deps
            and ordinary Rust deps.
    """

    # Compile protos
    name_pb = name + "_pb"
    name_fixed = name_pb + "_fixed"
    name_root = name + "_root"

    include_implicit_protobuf = kwargs.get("_implicit_protobuf", True)
    rust_deps = merge_rust_deps(
        kwargs.get("deps", []),
        kwargs.get("proto_deps", []),
        include_implicit_protobuf,
    )

    rust_grpc_compile(
        name = name_pb,
        crate_name = kwargs.get("crate_name", name),
        deps = rust_deps,
        **{
            k: v
            for (k, v) in kwargs.items()
            if k in proto_compile_attrs.keys() or
               k in bazel_build_rule_common_attrs or
               (k in rust_compile_attrs and k not in ["crate_name", "proto_deps", "deps"])
        }  # Forward args
    )

    # Rust plugin fragments are sibling files, not modules. After the isolated
    # plugin outputs are merged, wire matching serde/gRPC siblings into the base
    # module files before handing the tree to rust_library.
    rust_proto_crate_fixer(
        name = name_fixed,
        compilation = name_pb,
        deps = rust_deps,
    )

    rust_proto_crate_root(
        name = name_root,
        crate_dir = name_fixed,
    )

    name_rust_library = name + "_rust_library"
    common_attrs = {
        k: v
        for (k, v) in kwargs.items()
        if k in bazel_build_rule_common_attrs
    }
    inner_common_attrs = {
        k: v
        for (k, v) in common_attrs.items()
        if k not in ["visibility", "deprecation"]
    }

    # Create rust library
    rust_library(
        name = name_rust_library,
        crate_name = kwargs.get("crate_name", name),
        crate_root = name_root,
        edition = kwargs.get("edition", "2021"),
        srcs = [name_fixed],
        deps = [crate_label("prost")] +
               [crate_label("pbjson")] +
               [crate_label("serde")] +
               [crate_label("tonic"), crate_label("tonic-prost")] +
               rust_deps,
        proc_macro_deps = kwargs.get("proc_macro_deps", []) + [
            crate_label("prost-derive"),
        ],
        **inner_common_attrs
    )

    rust_proto_library_forward(
        name = name,
        actual = name_rust_library,
        compilation = name_pb,
        **common_attrs
    )
