package main

var rustCompileRuleTemplate = mustTemplate(`load(
    "@rules_proto_grpc//:defs.bzl",
    "ProtoPluginInfo",
    "proto_compile_attrs",
    "proto_compile_toolchains",
)
load(":common.bzl", "RustProtoInfo", "rust_proto_compile_impl")

# Create compile rule
{{ .Rule.Name }} = rule(
    implementation = rust_proto_compile_impl,
    attrs = dict(
        proto_compile_attrs,
        proto_deps = attr.label_list(
            providers = [RustProtoInfo],
            mandatory = False,
            doc = "Deprecated. Use deps instead. Other Rust proto targets that this proto directly depends upon. Used to generate extern_path options.",
        ),
        deps = attr.label_list(
            mandatory = False,
            doc = "Rust dependencies for this proto. Dependencies that provide RustProtoInfo are used to generate extern_path options; other Rust dependencies are ignored by codegen.",
        ),
        declared_proto_packages = attr.string_list(
            mandatory = True,
            doc = "List of proto packages that this rule generates Rust bindings for.",
        ),
        crate_name = attr.string(
            mandatory = False,
            doc = "Name of the Rust crate these protos will be compiled into later using rust_library.",
        ),
        _plugins = attr.label_list(
            providers = [ProtoPluginInfo],
            default = [{{ range .Rule.Plugins }}
                Label("{{ . }}"),{{ end }}
            ],
            cfg = "exec",
            doc = "List of protoc plugins to apply",
        ),
    ),
    toolchains = proto_compile_toolchains,
)`)

var rustLibraryRuleTemplateString = `load("@rules_proto_grpc//:defs.bzl", "bazel_build_rule_common_attrs", "proto_compile_attrs")
load("@rules_rust//rust:defs.bzl", "rust_library")
load(":common.bzl", "crate_label", "merge_rust_deps", "rust_compile_attrs", "rust_proto_library_forward")
load(":rust_fixer.bzl", "rust_proto_crate_fixer", "rust_proto_crate_root")
load(":{{ .Rule.Base }}_{{ .Rule.Kind }}_compile.bzl", "{{ .Rule.Base }}_{{ .Rule.Kind }}_compile")

def {{ .Rule.Name }}(name, **kwargs):
    """Generates Rust {{ .Rule.Kind }} code and wraps it in a rust_library.

    Args:
        name: Name of the generated rust_library target.
        **kwargs: Common Bazel attributes are forwarded to both generated
            targets; proto compile attributes are forwarded to
            {{ .Rule.Base }}_{{ .Rule.Kind }}_compile; Rust-specific attributes
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

    {{ .Rule.Base }}_{{ .Rule.Kind }}_compile(
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
`

var rustProtoLibraryRuleTemplate = mustTemplate(rustLibraryRuleTemplateString + `
    # Create {{ .Rule.Base }} library
    rust_library(
        name = name_rust_library,
        crate_name = kwargs.get("crate_name", name),
        crate_root = name_root,
        edition = kwargs.get("edition", "2021"),
        srcs = [name_fixed],
        deps = [crate_label("prost")] +
               [crate_label("pbjson")] +
               [crate_label("serde")] +
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
    )`)

var rustGrpcLibraryRuleTemplate = mustTemplate(rustLibraryRuleTemplateString + `
    # Create {{ .Rule.Base }} library
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
    )`)

var rustProtoCompileExampleTemplate = mustTemplate(`load("@rules_proto_grpc_{{ .Lang.Name }}//:defs.bzl", "{{ .Rule.Name }}")

{{ .Rule.Name }}(
    name = "person_{{ .Lang.Name }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    options = {
        "@rules_proto_grpc_rust//:rust_proto_plugin": ["type_attribute=.example.proto.Person=#[derive(Eq\\,Hash)]"],
    },
    protos = ["@rules_proto_grpc_example_protos//:person_proto"],
)

{{ .Rule.Name }}(
    name = "place_{{ .Lang.Name }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    options = {
        "@rules_proto_grpc_rust//:rust_proto_plugin": ["type_attribute=.example.proto.Place=#[derive(Eq\\,Hash)]"],
    },
    protos = ["@rules_proto_grpc_example_protos//:place_proto"],
)

{{ .Rule.Name }}(
    name = "thing_{{ .Lang.Name }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    protos = ["@rules_proto_grpc_example_protos//:thing_proto"],
)`)

var rustGrpcCompileExampleTemplate = mustTemplate(`load("@rules_proto_grpc_{{ .Lang.Name }}//:defs.bzl", "{{ .Rule.Name }}")

{{ .Rule.Name }}(
    name = "greeter_{{ .Lang.Name }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    protos = [
        "@rules_proto_grpc_example_protos//:greeter_grpc",
        "@rules_proto_grpc_example_protos//:thing_proto",
    ],
)`)

var rustProtoLibraryExampleTemplate = mustTemplate(`load("@rules_proto_grpc_{{ .Lang.Name }}//:defs.bzl", "{{ .Rule.Name }}")
load("@rules_rust//rust:defs.bzl", "rust_test")

{{ .Rule.Name }}(
    name = "common_types_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.common"],
    deps = [
        "@rules_proto_grpc_rust//google/type:money_rust_proto",
    ],
    protos = [
        "@rules_proto_grpc_example_protos//:common_types_proto",
    ],
)

{{ .Rule.Name }}(
    name = "person_place_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    options = {
        "@rules_proto_grpc_rust//:rust_proto_plugin": [
            "type_attribute=.example.proto.Person=#[derive(Eq\\,Hash)]",
            "type_attribute=.example.proto.Place=#[derive(Eq\\,Hash)]",
        ],
    },
    deps = [
        ":thing_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    ],
    protos = [
        "@rules_proto_grpc_example_protos//:person_proto",
        "@rules_proto_grpc_example_protos//:place_proto",
    ],
)

{{ .Rule.Name }}(
    name = "session_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.session"],
    deps = [
        "@rules_proto_grpc_rust//google/api:field_behavior_rust_proto",
        "@rules_proto_grpc_rust//google/api:resource_rust_proto",
        "@rules_proto_grpc_rust//google/type:latlng_rust_proto",
    ],
    protos = [
        "@rules_proto_grpc_example_protos//:session_proto",
    ],
)

{{ .Rule.Name }}(
    name = "event_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.events"],
    protos = [
        "@rules_proto_grpc_example_protos//:event_proto",
    ],
)

{{ .Rule.Name }}(
    name = "identity_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.events.identity.v1"],
    deps = [
        ":event_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    ],
    protos = [
        "@rules_proto_grpc_example_protos//:identity_proto",
    ],
)

{{ .Rule.Name }}(
    name = "thing_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    protos = [
        "@rules_proto_grpc_example_protos//:thing_proto",
    ],
)

rust_test(
    name = "proto_runtime_test",
    srcs = ["proto_runtime_test.rs"],
    deps = [
        ":person_place_{{ .Rule.Base }}_{{ .Rule.Kind }}",
        "@rules_proto_grpc_rust//rust:proto_runtime",
    ],
)

rust_test(
    name = "google_deps_test",
    srcs = ["google_deps_test.rs"],
    deps = [
        ":event_{{ .Rule.Base }}_{{ .Rule.Kind }}",
        ":identity_{{ .Rule.Base }}_{{ .Rule.Kind }}",
        ":session_{{ .Rule.Base }}_{{ .Rule.Kind }}",
        "@rules_proto_grpc_rust//google/api:field_behavior_rust_proto",
        "@rules_proto_grpc_rust//google/protobuf:protobuf_rust_proto",
        "@rules_proto_grpc_rust//google/type:latlng_rust_proto",
        "@rules_proto_grpc_rust//rust:proto_runtime",
    ],
)`)

var rustGrpcLibraryExampleTemplate = mustTemplate(`load("@rules_proto_grpc_{{ .Lang.Name }}//:defs.bzl", "rust_proto_library", "{{ .Rule.Name }}")

{{ .Rule.Name }}(
    name = "greeter_{{ .Rule.Base }}_{{ .Rule.Kind }}",
    declared_proto_packages = ["example.proto"],
    options = {
        "@rules_proto_grpc_rust//:rust_proto_plugin": [
            "type_attribute=.example.proto.Person=#[derive(Eq\\,Hash)]",
            "type_attribute=.example.proto.Place=#[derive(Eq\\,Hash)]",
        ],
    },
    deps = [
        ":thing_rust_proto",
    ],
    protos = [
        "@rules_proto_grpc_example_protos//:greeter_grpc",
        "@rules_proto_grpc_example_protos//:person_proto",
        "@rules_proto_grpc_example_protos//:place_proto",
    ],
)

rust_proto_library(
    name = "thing_rust_proto",
    declared_proto_packages = ["example.proto"],
    protos = [
        "@rules_proto_grpc_example_protos//:thing_proto",
    ],
)`)

var rustCompileRuleAttrs = append(append([]*Attr(nil), compileRuleAttrs...), []*Attr{
	&Attr{
		Name:      "declared_proto_packages",
		Type:      "string_list",
		Doc:       "List of proto packages that this rule generates Rust bindings for",
		Mandatory: true,
	},
	&Attr{
		Name:      "proto_deps",
		Type:      "label_list",
		Default:   "[]",
		Doc:       "Deprecated. Use ``deps`` instead. Other Rust proto targets that this proto directly depends upon",
		Mandatory: false,
		Providers: []string{"RustProtoInfo"},
	},
	&Attr{
		Name:      "deps",
		Type:      "label_list",
		Default:   "[]",
		Doc:       "Rust dependencies. Deps that provide ``RustProtoInfo`` are used for ``extern_path``; all deps are passed to the underlying ``rust_library`` by library rules",
		Mandatory: false,
	},
	&Attr{
		Name:      "crate_name",
		Type:      "string",
		Default:   "None",
		Doc:       "Name of the Rust crate these protos will be compiled into later using ``rust_library``",
		Mandatory: false,
	},
}...)

var rustLibraryRuleAttrs = append(append([]*Attr(nil), rustCompileRuleAttrs...), []*Attr{
	&Attr{
		Name:      "edition",
		Type:      "string",
		Default:   "2021",
		Doc:       "Rust edition to pass to underlying ``rust_library`` rule",
		Mandatory: false,
	},
	&Attr{
		Name:      "proc_macro_deps",
		Type:      "label_list",
		Default:   "[]",
		Doc:       "Additional labels to pass as proc_macro_deps attr to underlying ``rust_library`` rule",
		Mandatory: false,
	},
}...)

func makeRust() *Language {
	return &Language{
		Name:              "rust",
		DisplayName:       "Rust",
		Notes:             mustTemplate("Rules for generating Rust protobuf and gRPC ``.rs`` files and libraries. Libraries are created with ``rust_library`` from `rules_rust <https://github.com/bazelbuild/rules_rust>`_. Rust library rules use one public ``deps`` attribute for both generated Rust proto libraries and ordinary Rust crate dependencies. Generated proto dependencies provide package metadata that is converted into Prost ``extern_path`` options; ordinary Rust deps are passed through to ``rust_library`` and ignored for code generation. The legacy ``proto_deps`` attribute is still accepted as a deprecated compatibility alias and is merged into ``deps``.\n\nThe Rust module provides generated wrapper crates for common upstream protos under ``@rules_proto_grpc_rust//google/...``. In particular, ``@rules_proto_grpc_rust//google/protobuf:protobuf_rust_proto`` maps ``.google.protobuf`` to ``::google_protobuf::google::protobuf`` and is added implicitly to every ``rust_proto_library`` and ``rust_grpc_library``. Common Google API/type/rpc/longrunning protos should be listed through their generated Rust wrapper targets, for example ``@rules_proto_grpc_rust//google/api:field_behavior_rust_proto``, ``@rules_proto_grpc_rust//google/type:latlng_rust_proto``, or ``@rules_proto_grpc_rust//google/longrunning:operations_rust_proto``.\n\nGenerated Rust proto and gRPC libraries use the generated ``google_protobuf`` crate for ``google.protobuf`` types and do not depend on ``pbjson-types``. ``pbjson-types`` remains available only through ``@rules_proto_grpc_rust//rust:proto_runtime`` for application or test code that needs protobuf JSON string handling for well-known types while generated ``google_protobuf`` serde remains structural.\n\nRust library rules run a small post-merge fixup before calling ``rust_library``. The core rules execute each protoc plugin in an isolated action and then merge the plugin output trees. The Rust plugins emit sibling files such as ``foo.rs``, ``foo.serde.rs``, and ``foo.tonic.rs``; Rust does not compile those siblings unless the base module explicitly includes them. The fixup copies the merged tree and appends the required ``include!`` statements so generated serde and gRPC code is part of the crate.\n\nDownstream Rust code that needs to call ``prost`` or ``serde_json`` APIs on generated messages should depend on ``@rules_proto_grpc_rust//rust:proto_runtime``. That public target re-exports the exact ``prost``, ``prost-types``, ``pbjson``, ``pbjson-types``, ``proto-types``, ``serde``, and ``serde_json`` crate instances available from the Rust module, avoiding direct dependencies on the internal ``@rules_proto_grpc_rust_crates`` hub."),
		ModuleSuffixLines: `bazel_dep(name = "rules_rust", version = "0.69.0")`,
		SkipTestPlatforms: []string{"windows"},
		Rules: []*Rule{
			&Rule{
				Name:           "rust_proto_compile",
				Base:           "rust",
				Kind:           "proto",
				Implementation: rustCompileRuleTemplate,
				Plugins:        []string{"//:rust_proto_plugin", "//:rust_crate_plugin", "//:rust_serde_plugin"},
				BuildExample:   rustProtoCompileExampleTemplate,
				Doc:            "Generates Rust protobuf ``.rs`` files",
				Attrs:          rustCompileRuleAttrs,
			},
			&Rule{
				Name:           "rust_grpc_compile",
				Base:           "rust",
				Kind:           "grpc",
				Implementation: rustCompileRuleTemplate,
				Plugins:        []string{"//:rust_proto_plugin", "//:rust_crate_plugin", "//:rust_serde_plugin", "//:rust_grpc_plugin"},
				BuildExample:   rustGrpcCompileExampleTemplate,
				Doc:            "Generates Rust protobuf and gRPC ``.rs`` files",
				Attrs:          rustCompileRuleAttrs,
			},
			&Rule{
				Name:           "rust_proto_library",
				Base:           "rust",
				Kind:           "proto",
				Implementation: rustProtoLibraryRuleTemplate,
				BuildExample:   rustProtoLibraryExampleTemplate,
				Doc:            "Generates a Rust protobuf library using ``rust_library`` from ``rules_rust``",
				Attrs:          rustLibraryRuleAttrs,
			},
			&Rule{
				Name:           "rust_grpc_library",
				Base:           "rust",
				Kind:           "grpc",
				Implementation: rustGrpcLibraryRuleTemplate,
				BuildExample:   rustGrpcLibraryExampleTemplate,
				Doc:            "Generates a Rust protobuf and gRPC library using ``rust_library`` from ``rules_rust``",
				Attrs:          rustLibraryRuleAttrs,
			},
		},
	}
}
