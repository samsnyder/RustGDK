# SpatialOS C# Blank project

## Quick start

Build the project and start it with the default launch configuration:
```
spatial worker build
spatial local launch
```

Connects a new instance of the C# External worker to a running local
deployment.
```
spatial local worker launch External local
```

See sections below for more details.

## Troubleshooting

If you encounter any errors when trying to build or launch this project,
firstly make sure that all the dependencies of SpatialOS are met by running
`spatial diagnose`.

Warnings about missing files from `msbuild` during `spatial worker build` might
mean you don't have the required C++ build tools in your installation of
`msbuild` if it came with Visual Studio. You will need to modify your
installation of Visual Studio to include [C++ build tools](http://landinghub.visualstudio.com /visual-cpp-build-tools).

Visual Studio has a weird way to pick default configuration and platform for
builds. Because of that the default configuration when you open one of the
solutions for the first time in Visual Studio will be `DebugLinux`. This might
cause build errors on other platforms so use the dropdown in Visual Studio to
select a configuration matching your platform.

In case you need further assistance, don't hesitate to ask on the
[forums](https://forums.improbable.io/c/sup/setup-and-tutorials) and remember
to attach the contents of your `logs` folder and output from the failing
commands.

## Build

Use `spatial worker build` as usual.

The current worker configuration is using a modified version of the generated
`spatialos.csharp.build.json` which replaces `xbuild` with `msbuild` and adds
some extra steps. On old versions of Mono (before 5.0.0), even though it's best
to just update, you might want to replace `msbuild` with `xbuild`.

## Local launch

Use `spatial local launch` as usual.

Once the deloyment is running you can connect a worker with:

```
spatial local worker launch External local
```

`local` is an external launch configuration defined in
`spatialos.External.worker.json`. Feel free to change it or add more
configurations for different starting conditions.

When a worker connects the terminal which runs `spatial local launch` should
output several messages:

The first two confirm that the connection is successful.
```sh
[improbable.bridge.oracle.service.BridgeOracleInternalServiceImpl] The worker ExternalLocalWindows registered with SpatialOS successfully.

[improbable.bridge.logging.EngineLogMessageHandler] [Worker: ExternalLocalWindows] Successfully connected using the Receptionist -[WorkerLogger:Startup.cs]
```

You could also have configurations which start multiple workers or start
workers directly from an executable. Look at [External worker launch configuration](https://docs.improbable.io/reference/latest/workers/configuration/launch-configuration) for all the technical details.

## Attaching a debugger

The easiest option is using Visual Studio. Open the project properties and set
the command-line arguments in `Debug > Start options` to something like
`receptionist localhost 7777 ExternalDebug`. Then you can start the project
from Visual Studio. If you're using another IDE there must be a similar way to
configure and start the project with a visual debugger.

## Project structure

The C# workers which are in this project are called _External_ and _Managed_
to reflect the way they are configured and connected and their intended use as
an external worker which is often used to implement a game client or a managed
worker used for various tasks ranging from physics simulation to player login
to inventory management and microservices. Have a look at the [Glossary entry for Workers](https://docs.improbable.io/reference/latest/getting-started/concepts/glossary#worker) for a quick intro and links to learning
resources.

If you're building your own worker out of them, you're strongly encouraged to
replace the names with something meaningful for your use case. A simple Find
and Replace in both file contents and file names of the text "External" is
sufficient to rename the worker named "External" and keep the convention to use
the worker name in project files and build targets.

```
+-- schema/
+-- generated_code/csharp/
+-- dependencies/worker_sdk
+-- workers
    |-- External/
    |   |-- External/
    |   |-- BuildTargets.targets
    |   |-- External.targets
    |   |-- External.csproj
    |   |-- GeneratedCode.csproj
    |   |-- External.sln
    |   |-- spatialos.External.worker.json
    |   |-- spatialos_worker_packages.json
    |   |-- spatialos.csharp_msbuild.build.json
    |
    |-- Managed/
        |-- Managed/
        |-- BuildTargets.targets
        |-- Managed.targets
        |-- Managed.csproj
        |-- GeneratedCode.csproj
        |-- Managed.sln
        |-- spatialos.Managed.worker.json
        |-- spatialos_worker_packages.json
        |-- spatialos.csharp_msbuild.build.json
```

The SpatialOS C# Blank project contains two Visual Studio solutions each with
several C# projects.

  - `workers/External/External.sln` contains:
    - `External.csproj` with the worker sources
    - `GeneratedCode.csproj` with C# classes generated from schema sources

  - `workers/Managed/Managed.sln` follows the same structure as described for
    `External`.

### More about the worker project structure

The worker project `External.csproj` has its sources located in
`workers/External/External`, dependencies located in `dependencies/worker_sdk`,
and build targets for all platforms located in two places:

- `workers/External/External/bin/x64` for the worker executables
- `build/assembly/worker/External` for the packaged worker zips used by SpatialOS deployments

The `GeneratedCode.csproj` has its sources located in `generated_code/csharp`,
dependencies located also in `dependencies/worker_sdk`, and builds a
`GeneratedCode.dll` assembly for each configuration located in
`workers/External/External/bin/generated_code` and referenced by the worker
project.

### Why isn't there a single `GeneratedCode.csproj` shared by both solutions?

This might seem reasonable with the current state of the project since all C#
generated code is included in both projects. However, it's good to have the
flexibility to include only the code which is actually referenced by the worker
project. This results in smaller assemblies and faster build times when large
schemas are present.

## Cloud deployment

As usual set the `project_name` field in `spatialos.json` to match your SpatialOS project name. Then upload and launch:

```
spatial cloud upload <AssemblyName>
spatial cloud launch <AssemblyName> default_launch.json <deploymentname>
```

However, the launcher doesn't support using C# workers to start clients yet. You can still connect a client to the deployment by obtaining a login token from the launcher and passing it when starting the `cloud` worker external configuration.

```
spatial local worker launch External cloud <deploymentname> <login_token>
```

## Cross-platform builds

With the current build configuration for workers (defined in
`spatialos.csharp_msbuild.build.json`) running `spatial worker build` builds the
release configurations for each of the 3 supported platforms. You can build for
a specific platform only by passing the `target` flag:

```
spatial worker build --target=ReleaseWindows
```

This will skip all steps which have a target defined that is different from
"ReleaseWindows". The target string is case-insensitive.

If you want to be able to build the Debug configurations with `spatial` from
the command-line, it's easy to add more steps and define the respective
targets. You can already build and run all Debug configurations which are
defined for the projects from Visual Studio or by using `msbuild` on the
command-line.
