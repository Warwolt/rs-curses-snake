use winapi::shared::ntdef::LARGE_INTEGER;
use winapi::um::profileapi::{QueryPerformanceFrequency, QueryPerformanceCounter};

pub fn get_microsec_timestamp() -> i64 {
    let perf_counter_freq = get_perf_counter_freq() as i128;
    let current_ticks = get_perf_counter_ticks();
    // why can this overflow if we use i64? need to figure out that at some pointx
    let ticks_scaled_by_megahz = current_ticks as i128 * (1e6 as i128);
    let microsec_ticks = ticks_scaled_by_megahz / perf_counter_freq;
    microsec_ticks as i64
}

fn get_perf_counter_freq() -> i64 {
    unsafe {
        let mut perf_counter_freq = LARGE_INTEGER::default();
        QueryPerformanceFrequency(&mut perf_counter_freq);
        *perf_counter_freq.QuadPart()
    }
}

fn get_perf_counter_ticks() -> i64 {
    unsafe {
        let mut perf_counter_freq = LARGE_INTEGER::default();
        QueryPerformanceCounter(&mut perf_counter_freq);
        *perf_counter_freq.QuadPart()

    }
}
