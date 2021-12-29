# Contributing

Contributions are very welcome!

## Basics

ReaLearn is written in the programming language [Rust](https://www.rust-lang.org/). It makes heavy use of
[reaper-rs](https://github.com/helgoboss/reaper-rs), which provides Rust bindings for the
[REAPER C++ API](https://www.reaper.fm/sdk/plugin/plugin.php). _reaper-rs_ was developed together with ReaLearn
but is designed as independent library that can be used for REAPER plug-ins of all sorts.

Another noteworthy dependency and byproduct of ReaLearn is [helgoboss-learn](https://github.com/helgoboss/helgoboss-learn), a crate which
provides DAW-agnostic MIDI learn logic (basically the _source_ and _mode_ parts of ReaLearn). Like _reaper-rs_, it's
designed as independent library and could be used to provide similar MIDI-learn functionality in other DAWs.

## Directory structure

| Directory entry | Content                                                                                            |
| --------------- | ------------------------------------------------------------------------------------               |
| `/`             | Workspace root                                                                                     |
| `/api`          | Data structures of ReaLearn's preset API (which is e.g. used in the Lua-based ReaLearn Script)     |
| `/csi`          | Code for interfacing with the ControlSurfaceIntegrator (CSI) project                               |
| `/doc`          | Documentation                                                                                      |
| `/main`         | Main crate (`realearn`)                                                                            |
| `/proto`        | gRPC / Protocol Buffers schema for communicating with ReaLearn at runtime                          |
| `/rx-util`      | Utility crate providing primitives for reactive programming (`rx-util`)                            |
| `/swell-ui`     | Minimalistic UI framework based on [SWELL](https://www.cockos.com/wdl/) (`swell-ui`)               |
| `/resources`    | REAPER projects for manual testing, controller preset files                                        |

## Architecture

See [ARCHITECTURE](ARCHITECTURE.md).

## Build

### Windows

In the following you will find the complete instructions for Windows 10, including Rust setup. Points where you have to consider the target
architecture (REAPER 32-bit vs. 64-bit) are marked with :star:.

1. Setup "Build tools for Visual Studio 2019"
   - Rust uses native build toolchains. On Windows, it's necessary to use the MSVC (Microsoft Visual Studio
     C++) toolchain because REAPER plug-ins only work with that.
   - [Visual Studio downloads](https://visualstudio.microsoft.com/downloads/) → All downloads → Tools for Visual Studio 2019
     → Build Tools for Visual Studio 2019
   - Start it and follow the installer instructions
   - Required components
     - Workloads tab
       - "C++ build tools" (large box on the left)
       - Make sure "Windows 10 SDK" is checked on the right side (usually it is)
     - Language packs
       - English
2. Setup Rust
   - [Download](https://www.rust-lang.org/tools/install) and execute `rustup-init.exe`
   - Accept the defaults
   - Set the correct toolchain default :star:
     ```batch
     rustup default 1.55.0-x86_64-pc-windows-msvc
     ```
3. Download and install [Git for Windows](https://git-scm.com/download/win)
4. Clone the ReaLearn Git repository
   ```batch
   git clone --recurse-submodules https://github.com/helgoboss/realearn.git`
   cd realearn
   ```
5. Build ReaLearn (after that you should have a `realearn.dll` in `target\debug`)
   ```batch
   cargo build
   ```

### Linux

Complete instructions to build ReaLearn from a _fresh_ Ubuntu 18.04.3 LTS installation,
including Rust setup:

```sh
# Install native dependencies
sudo apt update
sudo apt install curl git build-essential pkg-config php nasm llvm-dev libclang-dev clang -y

# Install Rust (copied from the official Linux installation instructions)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # choose 1 (default)
source $HOME/.cargo/env

# Set the correct toolchain default
rustup default 1.55.0-x86_64-unknown-linux-gnu

# Clone ReaLearn repository (the "--recurse-submodules" is important!)
git clone --recurse-submodules https://github.com/helgoboss/realearn.git
cd realearn

# Build (after that you should have a "librealearn.so" in "target/debug")
cargo build
```

Some words about the native dependencies:

- `curl`, `git`, `build-essential` and `pkg-config` are bare essentials.
- `php` is needed to translate the ReaLearn dialog resource file to C++ so it can be processed by the SWELL
  dialog generator. It's also necessary for generating the 64-bit EEL assembler code. All of this is the
  typical WDL C++ way of doing things, no Rust specifics here.
- `nasm` is needed for assembling the 64-bit EEL assembler code to produce `asm-nseel-x64.o`, which is
  necessary to make the custom [EEL](https://www.cockos.com/EEL2/) control and feedback transformations in ReaLearn's
  absolute mode work.
- `llvm-dev`, `libclang-dev` and `clang` are necessary for building with feature `generate` (to generate
  bindings to C).

### macOS

The following instructions include Rust setup. However, it's very well possible that some native toolchain setup
instructions are missing, because I don't have a bare macOS installation at my disposal. The Rust installation script
should provide you with the necessary instructions if something is missing.

```sh
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # choose 1 (default)
source $HOME/.cargo/env
rustup default 1.55.0-x86_64-apple-darwin

# Clone ReaLearn
cd Downloads
git clone --recurse-submodules https://github.com/helgoboss/realearn.git
cd realearn

# Build ReaLearn
cargo build
```

### All operating systems

Regenerate bindings (for dialog resource IDs and EEL functions):

```sh
cargo build --features generate
cargo fmt
```

On Linux and macOS, this also regenerates the files `realearn.rc_mac_dlg` and `realearn.rc_mac_menu` using PHP.
This needs to be done after adjusting the Windows dialog resource file. So the best is to use the `generate`
feature only on Linux/WSL, which regenerates resource ID bindings and executes PHP in one go, so the macOS and
Linux dialog files can not go out of sync! These files must be checked into the source code repository whenever changed. 

## GUI

The basic GUI skeleton is based on the Windows dialog resource file [msvc.rc](main/src/infrastructure/ui/msvc/msvc.rc). 
I use Visual C++ 2019 resource editor to edit this file as part of the solution 
[msvc.sln](main/src/infrastructure/ui/msvc/msvc.sln).

## Test

Yes, there are tests but there should be more. While ReaLearn's basic building blocks
[helgoboss-learn](https://github.com/helgoboss/helgoboss-learn) and [reaper-rs](https://github.com/helgoboss/reaper-rs)
are tested pretty thoroughly, ReaLearn itself has room for improvement in that aspect. There's a growing built-in 
integration test, launchable via action `[developer] ReaLearn: Run integration test`. In future, it would be nice to run
this integration test during continuous integration, just like in *reaper-rs*.

## Log

It's possible to make ReaLearn output log messages to `stdout` by setting the `REALEARN_LOG` environment variable,
e.g. to `debug,vst=info`. It follows [this](https://docs.rs/env_logger/0.8.2/env_logger/index.html) format. Beware
that e.g. on Windows, `stdout` is not shown, not even when executing REAPER from the command line. One way to make it
visible is to execute REAPER with a debugger.

## Metrics

It's possible to make ReaLearn collect execution metrics by building ReaLearn with the feature `realearn-meter` (the
default) and setting the environment variable `REALEARN_METER`. If this environment variable is set (value doesn't
matter), ReaLearn will continuously record histograms of control surface (`IReaperControlSurface`) method execution
times. Control surface metrics are the most relevant metrics for ReaLearn because the processing is done in control
surface methods for the most part. That also means ReaLearn's logic is largely executed in the main thread, not in the
audio thread - which is atypical for a VST plug-in. That's also why REAPER's built-in FX performance measuring is not
too interesting in case of ReaLearn because all it does in the audio thread is processing some MIDI messages.

Metrics will be exposed in the following ways:

- Whenever response times exceed a certain threshold, they will be logged to `stdout` at warn level.
- If the projection server is running, metrics will be exposed at `/realearn/metrics` in the popular
  [Prometheus](https://prometheus.io/) format. That's great for visualization.
    - Just add this to your `prometheus.yml` (you might need to adjust the port):
    
        ```yaml
        scrape_configs:
          - job_name: 'realearn'
            metrics_path: '/realearn/metrics'
            static_configs:
              - targets: ['localhost:39080']
        ```
    - If you don't have the environment variable set, this will show zeros only.

## Debug

### Obtain debug symbols

Debug symbols are stripped from release builds but stored as build artifact of the GitHub Actions "Create release"
workflow. If you want the symbols for a specific build, proceed as follows:

1. Open the [list of ReaLearn "Create release" workflows](https://github.com/helgoboss/realearn/actions?query=workflow%3A%22Create+release%22).
2. Use the branch filter to show all releases builds made for a specific version, e.g. "v1.11.0".
3. Click the desired workflow.
    - GitHub seems to do a fuzzy search, so if there are pre-releases (e.g. "v1.11.0-pre2"), you will see them, too.
    - In that case, just choose the latest one.
4. You will see a list of artifacts, one for each OS-architecture combination.
5. Download the one you need and unzip it.
    - You will find both the library file and the symbol file (e.g. `realearn.pdb` for a Windows build).
    
### Turn on complete backtraces

As soon as you have the debug symbols, you can make ReaLearn print full backtraces (including line number etc.)
in the REAPER ReaScript console. Here's how you do it.

#### Windows

1. Set the environment variable `_NT_ALT_SYMBOL_PATH` to some directory of your choice.
2. Copy the PDB file in there.
3. Fire up REAPER with ReaLearn an make it panic. You should see a complete backtrace now.

### Lookup symbols for symbol-less backtraces

The problem with release builds is that they don't contain debug symbols and therefore backtraces usually contain not
much more than memory addresses. Especially backtraces generated by Windows release builds leave a lot to be desired.

ReaLearn has a built-in REAPER action which attempts to look up symbol information for a given error report:
"ReaLearn: Resolve symbols from clipboard". Works on Windows only. To be used like this:

1. Make sure the PDB for the release build in question is on the search path (see section above).
1. Fire up an ReaLearn using exactly that release build.
3. Copy the error report to the clipboard.
4. Execute the action.

## Release

This serves mainly as checklist for ReaLearn's author.

1. Bump up the version number in [main/Cargo.toml](main/Cargo.toml).
    - Either to a prerelease (e.g. `2.0.0-pre1`) or a final release (e.g. `2.0.0`).
    - This is important for having the correct version number displayed in ReaLearn UI.
1. Build at least once via `cargo build`.
    - This updates `Cargo.lock` and is important for not having the `-dirty` display in ReaLearn UI.
1. Update the user guide if not done already.
1. Create a version tag via `git tag v2.0.0-pre1`.
1. Push via `git push origin master --tags`.
1. While GitHub Actions executes the release job, take care of the following.
    - Can only be done by @helgoboss because it needs access to the [helgoboss.org website repository](https://github.com/helgoboss/helgoboss-org).
    - If it's a prerelease, make sure we are on a prerelease cycle branch of the website repository.
    - Add a changelog entry in [data.yaml](https://github.com/helgoboss/helgoboss-org/blob/master/src/data/projects/realearn/data.yaml).
    - In `src/snippets/projects/realearn/repo`, enter `git checkout master` and `git pull` to pull the latest user
      guide changes.
    - Push via `git push origin HEAD` and wait until Netlify deployed the page.
    - All of the following stuff needs to be done using Netlify's branch preview if it's a prerelease!
    - Update [helgoboss ReaPack index](https://github.com/helgoboss/reaper-packages/blob/master/index.xml).  
        - Generate ReaLearn-only ReaPack index by requesting [/projects/realearn/reapack.txt](https://www.helgoboss.org/projects/realearn/reapack.txt).
        - Integrate the generated index by copying everything from `<category name="Extensions">` and pasting it to the
          [helgoboss ReaPack index](https://github.com/helgoboss/reaper-packages/blob/master/index.xml) without
          overwriting the preset categories on the top of the file.
        - Don't push the index yet!
    - Author a REAPER forum ReaLearn thread entry with help of [/projects/realearn/reaper-forum.txt](https://www.helgoboss.org/projects/realearn/reaper-forum.txt)
      but don't submit yet!
    - Download the user guide by requesting [/projects/realearn/user-guide](https://www.helgoboss.org/projects/realearn/user-guide).
    - Copy the corresponding changelog entry in markdown format by requesting [/projects/realearn/changelog.md](https://www.helgoboss.org/projects/realearn/changelog.md).
1. Once the release job has finished successfully, edit the not-yet-published release that has been created.
    - Paste the copied changelog entry to the release notes.
    - Manually add the previously downloaded user guide as release artifact named `realearn-user-guide.pdf`.
1. Publish the release.
1. Push the [helgoboss ReaPack index](https://github.com/helgoboss/reaper-packages/blob/master/index.xml).
1. Submit the REAPER forum ReaLearn thread entry.
1. Check if synchronization of the ReaPack repository works.