:author: rules_proto_grpc
:description: rules_proto_grpc Bazel rules for Rust
:keywords: Bazel, Protobuf, gRPC, Protocol Buffers, Rules, Build, Starlark, Rust


Rust
====

Rules for generating Rust protobuf and gRPC ``.rs`` files and libraries. Libraries are created with ``rust_library`` from `rules_rust <https://github.com/bazelbuild/rules_rust>`_. Rust library rules use one public ``deps`` attribute for both generated Rust proto libraries and ordinary Rust crate dependencies. Generated proto dependencies provide package metadata that is converted into Prost ``extern_path`` options; ordinary Rust deps are passed through to ``rust_library`` and ignored for code generation. The legacy ``proto_deps`` attribute is still accepted as a deprecated compatibility alias and is merged into ``deps``.

The Rust module provides generated wrapper crates for common upstream protos under ``@rules_proto_grpc_rust//google/...``. In particular, ``@rules_proto_grpc_rust//google/protobuf:protobuf_rust_proto`` maps ``.google.protobuf`` to ``::google_protobuf::google::protobuf`` and is added implicitly to every ``rust_proto_library`` and ``rust_grpc_library``. Common Google API/type/rpc/longrunning protos should be listed through their generated Rust wrapper targets, for example ``@rules_proto_grpc_rust//google/api:field_behavior_rust_proto``, ``@rules_proto_grpc_rust//google/type:latlng_rust_proto``, or ``@rules_proto_grpc_rust//google/longrunning:operations_rust_proto``.

Generated Rust proto and gRPC libraries use the generated ``google_protobuf`` crate for ``google.protobuf`` types and do not depend on ``pbjson-types``. ``pbjson-types`` remains available only through ``@rules_proto_grpc_rust//rust:proto_runtime`` for application or test code that needs protobuf JSON string handling for well-known types while generated ``google_protobuf`` serde remains structural.

Rust library rules run a small post-merge fixup before calling ``rust_library``. The core rules execute each protoc plugin in an isolated action and then merge the plugin output trees. The Rust plugins emit sibling files such as ``foo.rs``, ``foo.serde.rs``, and ``foo.tonic.rs``; Rust does not compile those siblings unless the base module explicitly includes them. The fixup copies the merged tree and appends the required ``include!`` statements so generated serde and gRPC code is part of the crate.

Downstream Rust code that needs to call ``prost`` or ``serde_json`` APIs on generated messages should depend on ``@rules_proto_grpc_rust//rust:proto_runtime``. That public target re-exports the exact ``prost``, ``prost-types``, ``pbjson``, ``pbjson-types``, ``proto-types``, ``serde``, and ``serde_json`` crate instances available from the Rust module, avoiding direct dependencies on the internal ``@rules_proto_grpc_rust_crates`` hub.

.. list-table:: Rules
   :widths: 1 2
   :header-rows: 1

   * - Rule
     - Description
   * - `rust_proto_compile`_
     - Generates Rust protobuf ``.rs`` files
   * - `rust_grpc_compile`_
     - Generates Rust protobuf and gRPC ``.rs`` files
   * - `rust_proto_library`_
     - Generates a Rust protobuf library using ``rust_library`` from ``rules_rust``
   * - `rust_grpc_library`_
     - Generates a Rust protobuf and gRPC library using ``rust_library`` from ``rules_rust``

Installation
------------

The Rust module can be installed by adding the following lines to your MODULE.bazel file, replacing the version number placeholder with the desired version:

.. code-block:: python

   bazel_dep(name = "rules_proto_grpc_rust", version = "<version number here>")
   bazel_dep(name = "rules_rust", version = "0.69.0")

.. _rust_proto_compile:

rust_proto_compile
------------------

Generates Rust protobuf ``.rs`` files

Example
*******

Full example project can be found `here <https://github.com/rules-proto-grpc/rules_proto_grpc/tree/master/examples/rust/rust_proto_compile>`__

``BUILD.bazel``
^^^^^^^^^^^^^^^

.. code-block:: python

   load("@rules_proto_grpc_rust//:defs.bzl", "rust_proto_compile")
   
   rust_proto_compile(
       name = "person_rust_proto",
       declared_proto_packages = ["example.proto"],
       options = {
           "@rules_proto_grpc_rust//:rust_proto_plugin": ["type_attribute=.example.proto.Person=#[derive(Eq\\,Hash)]"],
       },
       protos = ["@rules_proto_grpc_example_protos//:person_proto"],
   )
   
   rust_proto_compile(
       name = "place_rust_proto",
       declared_proto_packages = ["example.proto"],
       options = {
           "@rules_proto_grpc_rust//:rust_proto_plugin": ["type_attribute=.example.proto.Place=#[derive(Eq\\,Hash)]"],
       },
       protos = ["@rules_proto_grpc_example_protos//:place_proto"],
   )
   
   rust_proto_compile(
       name = "thing_rust_proto",
       declared_proto_packages = ["example.proto"],
       protos = ["@rules_proto_grpc_example_protos//:thing_proto"],
   )

Attributes
**********

.. list-table:: Attributes for rust_proto_compile
   :widths: 1 1 1 1 4
   :header-rows: 1

   * - Name
     - Type
     - Mandatory
     - Default
     - Description
   * - ``protos``
     - ``label_list``
     - true
     - 
     - List of labels that provide the ``ProtoInfo`` provider (such as ``proto_library`` from ``@protobuf``)
   * - ``options``
     - ``string_list_dict``
     - false
     - ``[]``
     - Extra options to pass to plugins, as a dict of plugin label -> list of strings. The key * can be used exclusively to apply to all plugins
   * - ``verbose``
     - ``int``
     - false
     - ``0``
     - The verbosity level. Supported values and results are 0: Show nothing, 1: Show command, 2: Show command and sandbox after running protoc, 3: Show command and sandbox before and after running protoc, 4. Show env, command, expected outputs and sandbox before and after running protoc
   * - ``prefix_path``
     - ``string``
     - false
     - ``""``
     - Path to prefix to the generated files in the output directory
   * - ``extra_protoc_args``
     - ``string_list``
     - false
     - ``[]``
     - A list of extra command line arguments to pass directly to protoc, not as plugin options
   * - ``extra_protoc_files``
     - ``label_list``
     - false
     - ``[]``
     - List of labels that provide extra files to be available during protoc execution
   * - ``output_mode``
     - ``string``
     - false
     - ``PREFIXED``
     - The output mode for the target. PREFIXED (the default) will output to a directory named by the target within the current package root, NO_PREFIX will output directly to the current package. Using NO_PREFIX may lead to conflicting writes
   * - ``declared_proto_packages``
     - ``string_list``
     - true
     - 
     - List of proto packages that this rule generates Rust bindings for
   * - ``proto_deps``
     - ``label_list``
     - false
     - ``[]``
     - Deprecated. Use ``deps`` instead. Other Rust proto targets that this proto directly depends upon
   * - ``deps``
     - ``label_list``
     - false
     - ``[]``
     - Rust dependencies. Deps that provide ``RustProtoInfo`` are used for ``extern_path``; all deps are passed to the underlying ``rust_library`` by library rules
   * - ``crate_name``
     - ``string``
     - false
     - ``None``
     - Name of the Rust crate these protos will be compiled into later using ``rust_library``

Plugins
*******

- `@rules_proto_grpc_rust//:rust_proto_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__
- `@rules_proto_grpc_rust//:rust_crate_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__
- `@rules_proto_grpc_rust//:rust_serde_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__

.. _rust_grpc_compile:

rust_grpc_compile
-----------------

Generates Rust protobuf and gRPC ``.rs`` files

Example
*******

Full example project can be found `here <https://github.com/rules-proto-grpc/rules_proto_grpc/tree/master/examples/rust/rust_grpc_compile>`__

``BUILD.bazel``
^^^^^^^^^^^^^^^

.. code-block:: python

   load("@rules_proto_grpc_rust//:defs.bzl", "rust_grpc_compile")
   
   rust_grpc_compile(
       name = "greeter_rust_grpc",
       declared_proto_packages = ["example.proto"],
       protos = [
           "@rules_proto_grpc_example_protos//:greeter_grpc",
           "@rules_proto_grpc_example_protos//:thing_proto",
       ],
   )

Attributes
**********

.. list-table:: Attributes for rust_grpc_compile
   :widths: 1 1 1 1 4
   :header-rows: 1

   * - Name
     - Type
     - Mandatory
     - Default
     - Description
   * - ``protos``
     - ``label_list``
     - true
     - 
     - List of labels that provide the ``ProtoInfo`` provider (such as ``proto_library`` from ``@protobuf``)
   * - ``options``
     - ``string_list_dict``
     - false
     - ``[]``
     - Extra options to pass to plugins, as a dict of plugin label -> list of strings. The key * can be used exclusively to apply to all plugins
   * - ``verbose``
     - ``int``
     - false
     - ``0``
     - The verbosity level. Supported values and results are 0: Show nothing, 1: Show command, 2: Show command and sandbox after running protoc, 3: Show command and sandbox before and after running protoc, 4. Show env, command, expected outputs and sandbox before and after running protoc
   * - ``prefix_path``
     - ``string``
     - false
     - ``""``
     - Path to prefix to the generated files in the output directory
   * - ``extra_protoc_args``
     - ``string_list``
     - false
     - ``[]``
     - A list of extra command line arguments to pass directly to protoc, not as plugin options
   * - ``extra_protoc_files``
     - ``label_list``
     - false
     - ``[]``
     - List of labels that provide extra files to be available during protoc execution
   * - ``output_mode``
     - ``string``
     - false
     - ``PREFIXED``
     - The output mode for the target. PREFIXED (the default) will output to a directory named by the target within the current package root, NO_PREFIX will output directly to the current package. Using NO_PREFIX may lead to conflicting writes
   * - ``declared_proto_packages``
     - ``string_list``
     - true
     - 
     - List of proto packages that this rule generates Rust bindings for
   * - ``proto_deps``
     - ``label_list``
     - false
     - ``[]``
     - Deprecated. Use ``deps`` instead. Other Rust proto targets that this proto directly depends upon
   * - ``deps``
     - ``label_list``
     - false
     - ``[]``
     - Rust dependencies. Deps that provide ``RustProtoInfo`` are used for ``extern_path``; all deps are passed to the underlying ``rust_library`` by library rules
   * - ``crate_name``
     - ``string``
     - false
     - ``None``
     - Name of the Rust crate these protos will be compiled into later using ``rust_library``

Plugins
*******

- `@rules_proto_grpc_rust//:rust_proto_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__
- `@rules_proto_grpc_rust//:rust_crate_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__
- `@rules_proto_grpc_rust//:rust_serde_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__
- `@rules_proto_grpc_rust//:rust_grpc_plugin <https://github.com/rules-proto-grpc/rules_proto_grpc/blob/master/modules/rust/BUILD.bazel>`__

.. _rust_proto_library:

rust_proto_library
------------------

Generates a Rust protobuf library using ``rust_library`` from ``rules_rust``

Example
*******

Full example project can be found `here <https://github.com/rules-proto-grpc/rules_proto_grpc/tree/master/examples/rust/rust_proto_library>`__

``BUILD.bazel``
^^^^^^^^^^^^^^^

.. code-block:: python

   load("@rules_proto_grpc_rust//:defs.bzl", "rust_proto_library")
   load("@rules_rust//rust:defs.bzl", "rust_test")
   
   rust_proto_library(
       name = "common_types_rust_proto",
       declared_proto_packages = ["example.common"],
       deps = [
           "@rules_proto_grpc_rust//google/type:money_rust_proto",
       ],
       protos = [
           "@rules_proto_grpc_example_protos//:common_types_proto",
       ],
   )
   
   rust_proto_library(
       name = "person_place_rust_proto",
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
           "@rules_proto_grpc_example_protos//:person_proto",
           "@rules_proto_grpc_example_protos//:place_proto",
       ],
   )
   
   rust_proto_library(
       name = "session_rust_proto",
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
   
   rust_proto_library(
       name = "thing_rust_proto",
       declared_proto_packages = ["example.proto"],
       protos = [
           "@rules_proto_grpc_example_protos//:thing_proto",
       ],
   )
   
   rust_test(
       name = "proto_runtime_test",
       srcs = ["proto_runtime_test.rs"],
       deps = [
           ":person_place_rust_proto",
           "@rules_proto_grpc_rust//rust:proto_runtime",
       ],
   )
   
   rust_test(
       name = "google_deps_test",
       srcs = ["google_deps_test.rs"],
       deps = [
           ":session_rust_proto",
           "@rules_proto_grpc_rust//google/protobuf:protobuf_rust_proto",
           "@rules_proto_grpc_rust//google/type:latlng_rust_proto",
           "@rules_proto_grpc_rust//rust:proto_runtime",
       ],
   )

Attributes
**********

.. list-table:: Attributes for rust_proto_library
   :widths: 1 1 1 1 4
   :header-rows: 1

   * - Name
     - Type
     - Mandatory
     - Default
     - Description
   * - ``protos``
     - ``label_list``
     - true
     - 
     - List of labels that provide the ``ProtoInfo`` provider (such as ``proto_library`` from ``@protobuf``)
   * - ``options``
     - ``string_list_dict``
     - false
     - ``[]``
     - Extra options to pass to plugins, as a dict of plugin label -> list of strings. The key * can be used exclusively to apply to all plugins
   * - ``verbose``
     - ``int``
     - false
     - ``0``
     - The verbosity level. Supported values and results are 0: Show nothing, 1: Show command, 2: Show command and sandbox after running protoc, 3: Show command and sandbox before and after running protoc, 4. Show env, command, expected outputs and sandbox before and after running protoc
   * - ``prefix_path``
     - ``string``
     - false
     - ``""``
     - Path to prefix to the generated files in the output directory
   * - ``extra_protoc_args``
     - ``string_list``
     - false
     - ``[]``
     - A list of extra command line arguments to pass directly to protoc, not as plugin options
   * - ``extra_protoc_files``
     - ``label_list``
     - false
     - ``[]``
     - List of labels that provide extra files to be available during protoc execution
   * - ``output_mode``
     - ``string``
     - false
     - ``PREFIXED``
     - The output mode for the target. PREFIXED (the default) will output to a directory named by the target within the current package root, NO_PREFIX will output directly to the current package. Using NO_PREFIX may lead to conflicting writes
   * - ``declared_proto_packages``
     - ``string_list``
     - true
     - 
     - List of proto packages that this rule generates Rust bindings for
   * - ``proto_deps``
     - ``label_list``
     - false
     - ``[]``
     - Deprecated. Use ``deps`` instead. Other Rust proto targets that this proto directly depends upon
   * - ``deps``
     - ``label_list``
     - false
     - ``[]``
     - Rust dependencies. Deps that provide ``RustProtoInfo`` are used for ``extern_path``; all deps are passed to the underlying ``rust_library`` by library rules
   * - ``crate_name``
     - ``string``
     - false
     - ``None``
     - Name of the Rust crate these protos will be compiled into later using ``rust_library``
   * - ``edition``
     - ``string``
     - false
     - ``2021``
     - Rust edition to pass to underlying ``rust_library`` rule
   * - ``proc_macro_deps``
     - ``label_list``
     - false
     - ``[]``
     - Additional labels to pass as proc_macro_deps attr to underlying ``rust_library`` rule

.. _rust_grpc_library:

rust_grpc_library
-----------------

Generates a Rust protobuf and gRPC library using ``rust_library`` from ``rules_rust``

Example
*******

Full example project can be found `here <https://github.com/rules-proto-grpc/rules_proto_grpc/tree/master/examples/rust/rust_grpc_library>`__

``BUILD.bazel``
^^^^^^^^^^^^^^^

.. code-block:: python

   load("@rules_proto_grpc_rust//:defs.bzl", "rust_proto_library", "rust_grpc_library")
   
   rust_grpc_library(
       name = "greeter_rust_grpc",
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
   )

Attributes
**********

.. list-table:: Attributes for rust_grpc_library
   :widths: 1 1 1 1 4
   :header-rows: 1

   * - Name
     - Type
     - Mandatory
     - Default
     - Description
   * - ``protos``
     - ``label_list``
     - true
     - 
     - List of labels that provide the ``ProtoInfo`` provider (such as ``proto_library`` from ``@protobuf``)
   * - ``options``
     - ``string_list_dict``
     - false
     - ``[]``
     - Extra options to pass to plugins, as a dict of plugin label -> list of strings. The key * can be used exclusively to apply to all plugins
   * - ``verbose``
     - ``int``
     - false
     - ``0``
     - The verbosity level. Supported values and results are 0: Show nothing, 1: Show command, 2: Show command and sandbox after running protoc, 3: Show command and sandbox before and after running protoc, 4. Show env, command, expected outputs and sandbox before and after running protoc
   * - ``prefix_path``
     - ``string``
     - false
     - ``""``
     - Path to prefix to the generated files in the output directory
   * - ``extra_protoc_args``
     - ``string_list``
     - false
     - ``[]``
     - A list of extra command line arguments to pass directly to protoc, not as plugin options
   * - ``extra_protoc_files``
     - ``label_list``
     - false
     - ``[]``
     - List of labels that provide extra files to be available during protoc execution
   * - ``output_mode``
     - ``string``
     - false
     - ``PREFIXED``
     - The output mode for the target. PREFIXED (the default) will output to a directory named by the target within the current package root, NO_PREFIX will output directly to the current package. Using NO_PREFIX may lead to conflicting writes
   * - ``declared_proto_packages``
     - ``string_list``
     - true
     - 
     - List of proto packages that this rule generates Rust bindings for
   * - ``proto_deps``
     - ``label_list``
     - false
     - ``[]``
     - Deprecated. Use ``deps`` instead. Other Rust proto targets that this proto directly depends upon
   * - ``deps``
     - ``label_list``
     - false
     - ``[]``
     - Rust dependencies. Deps that provide ``RustProtoInfo`` are used for ``extern_path``; all deps are passed to the underlying ``rust_library`` by library rules
   * - ``crate_name``
     - ``string``
     - false
     - ``None``
     - Name of the Rust crate these protos will be compiled into later using ``rust_library``
   * - ``edition``
     - ``string``
     - false
     - ``2021``
     - Rust edition to pass to underlying ``rust_library`` rule
   * - ``proc_macro_deps``
     - ``label_list``
     - false
     - ``[]``
     - Additional labels to pass as proc_macro_deps attr to underlying ``rust_library`` rule
