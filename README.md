[![Crates.io](https://img.shields.io/crates/v/async-lease.svg)](https://crates.io/crates/async-lease)
[![Build Status](https://travis-ci.com/jonhoo/async-lease.svg?branch=master)](https://travis-ci.com/jonhoo/async-lease)

A command-line tool to restrict a process from using particular hardware
resources. It is particularly geared towards performance-sensitive
multi-core experiments, in which you may want to avoid or measure the
impact of CPU features like [SMT] (e.g., [Intel Hyper-Threading] or
[NUMA memory]. While this is possible with existing tools like
[`hwloc-bind`] and [`numactl`], it's often pretty inconvenient to get
the result you want. With `curb` on the other hand:

```console
$ curb --no-smt --no-numa mycommand
```

Will run `mycommand` _only_ on each physical core on a single NUMA node
(the first one specifically). Note that if you want to pass additional
arguments to the command under test, you can do so using `--`:

```console
$ curb --no-smt mycommand -- --no-crash --performance better
```

To visualize what CPUs are being used using [`lstopo`], do:

```console
$ curb --no-smt --no-numa lstopo -- --pid 0
```

That should show all active cores in green. If you don't have a window
manager running, add `-v` at the end and look for PUs marked with
`(running)`.

## Dependencies

Curb depends on the [`hwloc` crate], which in turn depends on the
[`hwloc` C library].

## OS Support

Curb currently only works on `cfg(unix)` systems, because it sets the
CPU binding [for the current process], and then calls [`exec`]. This
should be possible to work around by instead spawning the process,
getting its identifier, calling [`set_cpubind_for_process`], and then
waiting for the process to exit. However, this has a number of
downsides, like the process _initially_ being unconstrained and having
to proxy things like stdin, stdout, signals, and exit codes. Suggestions
for how to support non-UNIX platforms are warmly welcome!


  [SMT]: https://en.wikipedia.org/wiki/Simultaneous_multithreading
  [Intel Hyper-Threading]: https://en.wikipedia.org/wiki/Hyper-threading
  [NUMA memory]: https://en.wikipedia.org/wiki/Non-uniform_memory_access
  [`hwloc-bind`]: https://linux.die.net/man/1/hwloc-bind
  [`numactl`]: https://github.com/numactl/numactl
  [`lstopo`]: https://linux.die.net/man/1/lstopo
  [`hwloc` crate]: https://github.com/daschl/hwloc-rs
  [`hwloc` C library]: https://github.com/daschl/hwloc-rs#prerequisites
  [for the current process]: https://docs.rs/hwloc/0.5.0/hwloc/struct.Topology.html#method.set_cpubind
  [`exec`]: https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html#tymethod.exec
  [`set_cpubind_for_process`]: https://docs.rs/hwloc/0.5.0/hwloc/struct.Topology.html#method.set_cpubind_for_process
