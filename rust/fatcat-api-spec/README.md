# Rust API for fatcat

A scalable, versioned, API-oriented catalog of bibliographic entities and file metadata

## Overview
This client/server was generated by the [swagger-codegen]
(https://github.com/swagger-api/swagger-codegen) project.
By using the [OpenAPI-Spec](https://github.com/OAI/OpenAPI-Specification) from a remote server, you can easily generate a server stub.
-

To see how to make this your own, look here:

[README](https://github.com/swagger-api/swagger-codegen/blob/master/README.md)

- API version: 0.1.0
- Build date: 2018-09-23T00:19:26.675Z

This autogenerated project defines an API crate `fatcat` which contains:
* An `Api` trait defining the API in Rust.
* Data types representing the underlying data model.
* A `Client` type which implements `Api` and issues HTTP requests for each operation.
* A router which accepts HTTP requests and invokes the appropriate `Api` method for each operation.

It also contains an example server and client which make use of `fatcat`:
* The example server starts up a web server using the `fatcat` router,
  and supplies a trivial implementation of `Api` which returns failure for every operation.
* The example client provides a CLI which lets you invoke any single operation on the
  `fatcat` client by passing appropriate arguments on the command line.

You can use the example server and client as a basis for your own code.
See below for [more detail on implementing a server](#writing-a-server).


## Examples

Run examples with:

```
cargo run --example <example-name>
```

To pass in arguments to the examples, put them after `--`, for example:

```
cargo run --example client -- --help
```

### Running the server
To run the server, follow these simple steps:

```
cargo run --example server
```

### Running a client
To run a client, follow one of the following simple steps:

```
cargo run --example client CreateContainer
cargo run --example client CreateContainerBatch
cargo run --example client DeleteContainer
cargo run --example client GetContainer
cargo run --example client GetContainerHistory
cargo run --example client LookupContainer
cargo run --example client UpdateContainer
cargo run --example client CreateCreator
cargo run --example client CreateCreatorBatch
cargo run --example client DeleteCreator
cargo run --example client GetCreator
cargo run --example client GetCreatorHistory
cargo run --example client GetCreatorReleases
cargo run --example client LookupCreator
cargo run --example client UpdateCreator
cargo run --example client GetEditor
cargo run --example client GetEditorChangelog
cargo run --example client GetStats
cargo run --example client AcceptEditgroup
cargo run --example client CreateEditgroup
cargo run --example client GetChangelog
cargo run --example client GetChangelogEntry
cargo run --example client GetEditgroup
cargo run --example client CreateFile
cargo run --example client CreateFileBatch
cargo run --example client DeleteFile
cargo run --example client GetFile
cargo run --example client GetFileHistory
cargo run --example client LookupFile
cargo run --example client UpdateFile
cargo run --example client CreateRelease
cargo run --example client CreateReleaseBatch
cargo run --example client CreateWork
cargo run --example client DeleteRelease
cargo run --example client GetRelease
cargo run --example client GetReleaseFiles
cargo run --example client GetReleaseHistory
cargo run --example client LookupRelease
cargo run --example client UpdateRelease
cargo run --example client CreateWorkBatch
cargo run --example client DeleteWork
cargo run --example client GetWork
cargo run --example client GetWorkHistory
cargo run --example client GetWorkReleases
cargo run --example client UpdateWork
```

### HTTPS
The examples can be run in HTTPS mode by passing in the flag `--https`, for example:

```
cargo run --example server -- --https
```

This will use the keys/certificates from the examples directory. Note that the server chain is signed with
`CN=localhost`.


## Writing a server

The server example is designed to form the basis for implementing your own server. Simply follow these steps.

* Set up a new Rust project, e.g., with `cargo init --bin`.
* Insert `fatcat` into the `members` array under [workspace] in the root `Cargo.toml`, e.g., `members = [ "fatcat" ]`.
* Add `fatcat = {version = "0.1.0", path = "fatcat"}` under `[dependencies]` in the root `Cargo.toml`.
* Copy the `[dependencies]` and `[dev-dependencies]` from `fatcat/Cargo.toml` into the root `Cargo.toml`'s `[dependencies]` section.
  * Copy all of the `[dev-dependencies]`, but only the `[dependencies]` that are required by the example server. These should be clearly indicated by comments.
  * Remove `"optional = true"` from each of these lines if present.

Each autogenerated API will contain an implementation stub and main entry point, which should be copied into your project the first time:
```
cp fatcat/examples/server.rs src/main.rs
cp fatcat/examples/server_lib/mod.rs src/lib.rs
cp fatcat/examples/server_lib/server.rs src/server.rs
```

Now

* From `src/main.rs`, remove the `mod server_lib;` line, and uncomment and fill in the `extern crate` line with the name of this server crate.
* Move the block of imports "required by the service library" from `src/main.rs` to `src/lib.rs` and uncomment.
* Change the `let server = server::Server {};` line to `let server = SERVICE_NAME::server().unwrap();` where `SERVICE_NAME` is the name of the server crate.
* Run `cargo build` to check it builds.
* Run `cargo fmt` to reformat the code.
* Commit the result before making any further changes (lest format changes get confused with your own updates).

Now replace the implementations in `src/server.rs` with your own code as required.

## Updating your server to track API changes

Later, if the API changes, you can copy new sections  from the autogenerated API stub into your implementation.
Alternatively, implement the now-missing methods based on the compiler's error messages.
