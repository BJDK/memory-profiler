use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;

pub struct Opts {
    is_initialized: bool,

    pub base_server_port: u16,
    pub chown_output_to: Option< u32 >,
    pub disabled_by_default: bool,
    pub enable_broadcasts: bool,
    pub enable_server: bool,
    pub enable_shadow_stack: bool,
    pub grab_backtraces_on_free: bool,
    pub include_file: Option< String >,
    pub output_path_pattern: Cow< 'static, str >,
    pub register_sigusr1: bool,
    pub register_sigusr2: bool,
    pub use_perf_event_open: bool,
    pub write_binaries_to_output: bool,
    pub zero_memory: bool,
    pub gather_mmap_calls: bool,
    pub backtrace_cache_size_level_1: usize,
    pub backtrace_cache_size_level_2: usize,
    pub cull_temporary_allocations: bool,
    pub temporary_allocation_lifetime_threshold: u64,
    pub temporary_allocation_pending_threshold: Option< usize >
}

static mut OPTS: Opts = Opts {
    is_initialized: false,

    base_server_port: 8100,
    chown_output_to: None,
    disabled_by_default: false,
    enable_broadcasts: false,
    enable_server: false,
    enable_shadow_stack: true,
    grab_backtraces_on_free: true,
    include_file: None,
    output_path_pattern: Cow::Borrowed( "memory-profiling_%e_%t_%p.dat" ),
    register_sigusr1: true,
    register_sigusr2: true,
    use_perf_event_open: true,
    write_binaries_to_output: true,
    zero_memory: false,
    gather_mmap_calls: false,
    backtrace_cache_size_level_1: 16 * 1024,
    backtrace_cache_size_level_2: 320 * 1024,
    cull_temporary_allocations: false,
    temporary_allocation_lifetime_threshold: 10000,
    temporary_allocation_pending_threshold: None,
};

trait ParseVar: Sized {
    fn parse_var( value: &OsStr ) -> Option< Self >;
}

impl ParseVar for bool {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        Some( value == "1" || value == "true" )
    }
}

impl ParseVar for u16 {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        value.to_str()?.parse().ok()
    }
}

impl ParseVar for u32 {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        value.to_str()?.parse().ok()
    }
}

impl ParseVar for u64 {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        value.to_str()?.parse().ok()
    }
}

impl ParseVar for usize {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        value.to_str()?.parse().ok()
    }
}

impl ParseVar for String {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        value.to_str().map( |value| value.into() )
    }
}

impl< 'a > ParseVar for Cow< 'a, str > {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        let value = String::parse_var( value )?;
        Some( value.into() )
    }
}

impl< T > ParseVar for Option< T > where T: ParseVar {
    fn parse_var( value: &OsStr ) -> Option< Self > {
        if let Some( value ) = T::parse_var( value ) {
            Some( Some( value ) )
        } else {
            None
        }
    }
}

macro_rules! opts {
    ($($name:expr => $var:expr),+) => {{
        $(
            let var = $var;
            let name = $name;
            *var = env::var_os( $name )
                .and_then( |value| ParseVar::parse_var( &value ) )
                .unwrap_or( (*var).clone() );

            info!( "    {:40} = {:?}", name, *var );
        )+
    }}
}

pub unsafe fn initialize() {
    info!( "Options:" );

    let opts = &mut OPTS;
    opts! {
        "MEMORY_PROFILER_BASE_SERVER_PORT"          => &mut opts.base_server_port,
        "MEMORY_PROFILER_CHOWN_OUTPUT_TO"           => &mut opts.chown_output_to,
        "MEMORY_PROFILER_DISABLE_BY_DEFAULT"        => &mut opts.disabled_by_default,
        "MEMORY_PROFILER_ENABLE_BROADCAST"          => &mut opts.enable_broadcasts,
        "MEMORY_PROFILER_ENABLE_SERVER"             => &mut opts.enable_server,
        "MEMORY_PROFILER_GRAB_BACKTRACES_ON_FREE"   => &mut opts.grab_backtraces_on_free,
        "MEMORY_PROFILER_INCLUDE_FILE"              => &mut opts.include_file,
        "MEMORY_PROFILER_OUTPUT"                    => &mut opts.output_path_pattern,
        "MEMORY_PROFILER_REGISTER_SIGUSR1"          => &mut opts.register_sigusr1,
        "MEMORY_PROFILER_REGISTER_SIGUSR2"          => &mut opts.register_sigusr2,
        "MEMORY_PROFILER_USE_PERF_EVENT_OPEN"       => &mut opts.use_perf_event_open,
        "MEMORY_PROFILER_USE_SHADOW_STACK"          => &mut opts.enable_shadow_stack,
        "MEMORY_PROFILER_WRITE_BINARIES_TO_OUTPUT"  => &mut opts.write_binaries_to_output,
        "MEMORY_PROFILER_ZERO_MEMORY"               => &mut opts.zero_memory,
        "MEMORY_PROFILER_GATHER_MMAP_CALLS"         => &mut opts.gather_mmap_calls,
        "MEMORY_PROFILER_BACKTRACE_CACHE_SIZE_LEVEL_1"
            => &mut opts.backtrace_cache_size_level_1,
        "MEMORY_PROFILER_BACKTRACE_CACHE_SIZE_LEVEL_2"
            => &mut opts.backtrace_cache_size_level_2,
        "MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS"
            => &mut opts.cull_temporary_allocations,
        "MEMORY_PROFILER_TEMPORARY_ALLOCATION_LIFETIME_THRESHOLD"
            => &mut opts.temporary_allocation_lifetime_threshold,
        "MEMORY_PROFILER_TEMPORARY_ALLOCATION_PENDING_THRESHOLD"
            => &mut opts.temporary_allocation_pending_threshold
    }

    opts.is_initialized = true;
}

#[inline]
pub fn get() -> &'static Opts {
    let opts = unsafe { &OPTS };
    debug_assert!( opts.is_initialized );

    opts
}

#[inline]
pub fn crosscheck_unwind_results_with_libunwind() -> bool {
    false
}

pub fn emit_partial_backtraces() -> bool {
    if !cfg!(debug_assertions) {
        return true;
    }

    lazy_static! {
        static ref VALUE: bool = {
            let value = env::var_os( "MEMORY_PROFILER_EMIT_PARTIAL_BACKTRACES" )
                .map( |value| value == "1" )
                .unwrap_or( true );

            if value {
                info!( "Will emit partial backtraces" );
            } else {
                info!( "Will NOT emit partial backtraces" );
            }

            value
        };
    }

    *VALUE
}
