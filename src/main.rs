use rand::seq::SliceRandom;
use rand::Rng;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{exit, Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(author = "")]
struct Opt {
    /// Enable verbose debug output.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Disable simultaneous multithreading (SMT) technologies like Intel Hyper-Threading.
    #[structopt(long = "no-smt")]
    disable_smt: bool,

    /// Disable NUMA by forcing the command to run on only a single numa node.
    #[structopt(long = "no-numa")]
    disable_numa: bool,

    /// Run on at most this many cores.
    #[structopt(short = "n", long = "ncores")]
    ncores: Option<usize>,

    /// Randomize all selections (like choice of HT, NUMA node, or cores).
    #[structopt(short = "r", long = "randomize")]
    randomize: bool,

    /// The command to run.
    #[structopt(parse(from_os_str))]
    command: PathBuf,

    /// Argument to pass to the given command.
    ///
    /// You can use `--` in the argument list to pass arguments that would otherwise be interpreted
    /// by curb. For example, if you want to pass `-v` to `command`, use `curb mybin -- -v`.
    arguments: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let mut cmd = Command::new(&opt.command);
    cmd.args(&opt.arguments);

    // Before we exec, we need to set the right binds
    let flags = hwloc::CPUBIND_PROCESS | hwloc::CPUBIND_STRICT;
    let mut topo = hwloc::Topology::new();
    let mut allowed = topo.get_cpubind(flags).unwrap();
    let mut rng = rand::thread_rng();

    if opt.disable_smt {
        // clear bits for all but the first PU on each core
        for core in topo.objects_with_type(&hwloc::ObjectType::Core).unwrap() {
            if opt.verbose {
                eprintln!("found core #{}", core.logical_index());
            }

            let mut pus = core.children();
            pus.retain(|pu| pu.object_type() == hwloc::ObjectType::PU);
            let select = if opt.randomize {
                rng.gen_range(0, pus.len())
            } else {
                0
            };

            for (i, pu) in pus.into_iter().enumerate() {
                if i == select {
                    continue;
                }
                if opt.verbose {
                    eprintln!("disabling SMT PU#{}", pu.os_index());
                }
                allowed.unset(pu.os_index());
            }
        }
    }

    if opt.disable_numa {
        let mut numas = topo
            .objects_with_type(&hwloc::ObjectType::NUMANode)
            .unwrap();
        if opt.randomize {
            numas.shuffle(&mut rng);
        }

        for numa in numas.into_iter().skip(1) {
            if opt.verbose {
                eprintln!("found extra NUMA node #{}", numa.sibling_rank());
            }

            for cpu in numa.cpuset().unwrap() {
                if opt.verbose {
                    eprintln!("disabling extra NUMA PU");
                }
                allowed.unset(cpu);
            }
        }
    }

    if let Some(ncores) = opt.ncores {
        let mut all: Vec<_> = allowed.clone().into_iter().collect();
        if opt.randomize {
            all.shuffle(&mut rng);
        }

        for n in all.into_iter().skip(ncores) {
            allowed.unset(n);
        }
    }

    if let Err(e) = topo.set_cpubind(allowed, flags) {
        match e {
            hwloc::CpuBindError::Generic(code, msg) => {
                eprintln!("Could not bind hardware resources: {} ({})", msg, code);
                exit(1);
            }
        }
    }

    let e = cmd.exec();
    if e.kind() == io::ErrorKind::NotFound {
        eprintln!("Unknown command {}", opt.command.display());
        exit(127);
    } else if e.kind() == io::ErrorKind::PermissionDenied {
        eprintln!(
            "The file '{}' is not executable by this user",
            opt.command.display()
        );
        exit(126);
    } else {
        eprintln!("Could not execute {}: {}", opt.command.display(), e);
        exit(125);
    }
}
