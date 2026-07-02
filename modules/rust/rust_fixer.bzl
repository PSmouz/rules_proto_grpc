"""Helpers for assembling Rust crates from isolated protoc plugin output.

rules_proto_grpc runs each protoc plugin in a separate action and then merges
the resulting output trees. The Rust plugin stack emits sibling files such as
`foo.rs`, `foo.serde.rs`, and `foo.tonic.rs`; Rust does not discover those
siblings automatically. The fixer rule copies the merged tree and appends the
needed `include!` statements so serde and gRPC code is part of the crate.
"""

load("@rules_proto_grpc//:defs.bzl", "ProtoCompileInfo")
load(
    ":common.bzl",
    "RUST_RAW_IDENTIFIER_KEYWORDS",
    "RustProtoInfo",
    "proto_package_is_ancestor_or_equal",
    "rust_proto_package_path",
)

def _rust_path_for_proto_package(crate_name, package):
    return "::{}::{}::".format(crate_name, rust_proto_package_path(package))

def _ancestor_relative_rewrite_specs(compilation, deps):
    """Builds prefix rewrite specs for ancestor-package Rust imports.

    Prost's extern_path matching is package-prefix based. If a dependency
    declares `example.events` and the current crate declares
    `example.events.identity.v1`, externing `example.events` would also match
    the current crate's own package. The compile rule intentionally skips that
    unsafe extern. Prost then emits relative Rust paths back to the imported
    ancestor package, so the fixer rewrites just those ancestor-relative prefixes
    to the dependency crate path after codegen.
    """
    declared_packages = compilation[RustProtoInfo].declared_proto_packages
    specs = []
    seen = {}

    for dep in deps:
        if RustProtoInfo not in dep:
            continue

        dep_info = dep[RustProtoInfo]
        for dep_package in dep_info.declared_proto_packages:
            dep_parts = dep_package.split(".")
            for declared_package in declared_packages:
                if declared_package == dep_package:
                    continue
                if not proto_package_is_ancestor_or_equal(dep_package, declared_package):
                    continue

                super_count = len(declared_package.split(".")) - len(dep_parts)
                if super_count <= 0:
                    continue

                declared_path = declared_package.replace(".", "/")
                prefix = "super::" * super_count
                replacement = _rust_path_for_proto_package(dep_info.crate_name, dep_package)
                spec_key = "{}|{}|{}".format(declared_path, prefix, replacement)
                if spec_key in seen:
                    continue
                seen[spec_key] = True

                # The shell action sorts by this key so deeper relative paths
                # run first and cannot be partially rewritten by shallower ones.
                specs.append("{}|{}".format(999 - super_count, spec_key))

    return specs

def _rust_proto_crate_root(ctx):
    """Writes a crate root that includes the fixed Rust output tree.

    Args:
        ctx: Rule context.

    Returns:
        A `DefaultInfo` provider containing the generated crate root file.
    """
    name = ctx.attr.crate_dir
    lib_rs = ctx.actions.declare_file("%s_lib.rs" % name)
    ctx.actions.write(
        lib_rs,
        '#![allow(clippy::all)]\ninclude!("%s/mod.rs");' % name,
        False,
    )
    return [DefaultInfo(
        files = depset([lib_rs]),
    )]

def _rust_proto_crate_fixer(ctx):
    """Adds module includes for sibling Rust files emitted by plugins.

    The core merger materializes tree artifacts as symlinks, so the copy must
    dereference links before editing. Each base `*.rs` file is updated to
    include matching `*.serde.rs` and `*.tonic.rs` siblings when they exist.

    Args:
        ctx: Rule context.

    Returns:
        A `DefaultInfo` provider containing the fixed output tree.
    """
    compilation = ctx.attr.compilation[ProtoCompileInfo]
    rewrite_specs = _ancestor_relative_rewrite_specs(ctx.attr.compilation, ctx.attr.deps)
    keyword_rewrites = [
        keyword
        for keyword in RUST_RAW_IDENTIFIER_KEYWORDS
    ]
    in_dir = compilation.output_dirs.to_list()[0]
    out_dir = ctx.actions.declare_directory("%s_fixed" % compilation.label.name)

    ctx.actions.run_shell(
        outputs = [out_dir],
        inputs = [in_dir],
        arguments = [in_dir.path, out_dir.path] + rewrite_specs + ["--"] + keyword_rewrites,
        command = """
set -eu

out_dir="$2"

cp -RL "$1"/. "$2"/
chmod -R +w "$2"

find "$2" -type f ! -name 'mod.rs' ! -name '*.serde.rs' ! -name '*.tonic.rs' | while read -r base; do
    dir="$(dirname "$base")"
    module="$(basename "${base%.rs}")"
    for generated in "$dir/$module".serde.rs "$dir/$module".tonic.rs; do
        if [ -f "$generated" ]; then
            printf 'include!("%s");\n' "$(basename "$generated")" >> "$base"
        fi
    done
done

shift 2
ancestor_specs=""
while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
    ancestor_specs="${ancestor_specs}
$1"
    shift
done

if [ "$#" -gt 0 ]; then
    shift
fi

printf '%s\n' "$ancestor_specs" | sort | while read -r spec; do
    if [ -z "$spec" ]; then
        continue
    fi

    rest="${spec#*|}"
    declared_path="${rest%%|*}"
    rest="${rest#*|}"
    prefix="${rest%%|*}"
    replacement="${rest#*|}"
    package_dir="$out_dir/$declared_path"

    if [ -d "$package_dir" ]; then
        find "$package_dir" -type f -name '*.rs' | while read -r generated; do
            sed -i.bak "s#${prefix}#${replacement}#g" "$generated"
            rm -f "$generated.bak"
        done
    fi
done

for keyword in "$@"; do
    find "$out_dir" -type f -name '*.rs' | while read -r generated; do
        sed -i.bak "s|::${keyword}::|::r#${keyword}::|g" "$generated"
        rm -f "$generated.bak"
    done
done
""",
    )

    return [DefaultInfo(
        files = depset([out_dir]),
    )]

rust_proto_crate_root = rule(
    doc = "Writes the crate root used by Rust proto and gRPC library rules.",
    implementation = _rust_proto_crate_root,
    attrs = {
        "crate_dir": attr.string(
            doc = "Directory containing the fixed generated Rust module tree.",
            mandatory = True,
        ),
    },
)

rust_proto_crate_fixer = rule(
    doc = "Copies generated Rust output and wires serde/gRPC sibling files into each module.",
    implementation = _rust_proto_crate_fixer,
    attrs = {
        "compilation": attr.label(
            doc = "Rust proto compile target whose output tree should be fixed.",
            providers = [ProtoCompileInfo, RustProtoInfo],
            mandatory = True,
        ),
        "deps": attr.label_list(
            doc = "Rust dependencies used to repair ancestor-package relative imports.",
        ),
    },
)
