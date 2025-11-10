//! Performance Validation
//!
//! Provides RDTSC benchmarking and tick measurement utilities for hot path validation.
//! Ensures operations meet the Chatman Constant (≤8 ticks = 2ns budget).

use thiserror::Error;

/// Performance validation error
#[derive(Error, Debug)]
pub enum PerformanceValidationError {
    /// Tick budget exceeded
    #[error("Tick budget exceeded: {0} > {1} (Chatman Constant violation)")]
    TickBudgetExceeded(u64, u64),
    /// Invalid measurement
    #[error("Invalid measurement: {0}")]
    InvalidMeasurement(String),
    /// Measurement failed
    #[error("Measurement failed: {0}")]
    MeasurementFailed(String),
}

/// Result type for performance validation
pub type PerformanceValidationResult<T> = Result<T, PerformanceValidationError>;

/// Tick budget for hot path operations (Chatman Constant: 8 ticks = 2ns)
pub const HOT_PATH_TICK_BUDGET: u64 = 8;

/// Tick counter using RDTSC (Read Time-Stamp Counter)
///
/// On x86_64, uses `rdtsc` instruction for cycle counting.
/// On other platforms, uses `std::time::Instant` as fallback.
pub struct TickCounter {
    /// Start tick count
    start_ticks: u64,
}

impl TickCounter {
    /// Create a new tick counter and start counting
    pub fn start() -> Self {
        Self {
            start_ticks: Self::read_ticks(),
        }
    }

    /// Read current tick count
    fn read_ticks() -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { std::arch::x86_64::_rdtsc() }
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64: Use CNTVCT_EL0 (Virtual Count Register)
            let val: u64;
            unsafe {
                std::arch::asm!(
                    "mrs {}, cntvct_el0",
                    out(reg) val,
                    options(nostack, nomem)
                );
            }
            val
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Fallback: use SystemTime for non-x86_64/ARM64 platforms
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        }
    }

    /// Get elapsed ticks since start
    pub fn elapsed_ticks(&self) -> u64 {
        Self::read_ticks().saturating_sub(self.start_ticks)
    }

    /// Check if elapsed ticks exceed budget
    pub fn exceeds_budget(&self, budget: u64) -> bool {
        self.elapsed_ticks() > budget
    }

    /// Assert that elapsed ticks are within budget
    pub fn assert_within_budget(&self, budget: u64) -> PerformanceValidationResult<()> {
        let elapsed = self.elapsed_ticks();
        if elapsed > budget {
            Err(PerformanceValidationError::TickBudgetExceeded(
                elapsed, budget,
            ))
        } else {
            Ok(())
        }
    }

    /// Assert that elapsed ticks are within hot path budget (≤8 ticks)
    pub fn assert_within_hot_path_budget(&self) -> PerformanceValidationResult<()> {
        self.assert_within_budget(HOT_PATH_TICK_BUDGET)
    }
}

/// Measure ticks for a closure
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::performance::measure_ticks;
///
/// let (result, ticks) = measure_ticks(|| {
///     // Hot path operation
///     hot_path_operation()
/// });
///
/// assert!(ticks <= 8, "Exceeded tick budget: {} > 8", ticks);
/// ```
pub fn measure_ticks<F, T>(f: F) -> (T, u64)
where
    F: FnOnce() -> T,
{
    let counter = TickCounter::start();
    let result = f();
    let ticks = counter.elapsed_ticks();
    (result, ticks)
}

/// Measure ticks for an async operation
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::performance::measure_ticks_async;
///
/// let (result, ticks) = measure_ticks_async(async || {
///     // Async hot path operation
///     async_hot_path_operation().await
/// }).await;
///
/// assert!(ticks <= 8, "Exceeded tick budget: {} > 8", ticks);
/// ```
#[cfg(feature = "async")]
pub async fn measure_ticks_async<F, Fut, T>(f: F) -> (T, u64)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let counter = TickCounter::start();
    let result = f().await;
    let ticks = counter.elapsed_ticks();
    (result, ticks)
}

/// Zero-cost abstraction for tick measurement
///
/// This wrapper provides a zero-cost abstraction over tick measurement,
/// allowing for flexible measurement strategies without runtime overhead.
pub struct TickMeasurer<F> {
    /// Function to measure
    f: F,
}

impl<F> TickMeasurer<F> {
    /// Create a new tick measurer
    pub fn new(f: F) -> Self {
        Self { f }
    }

    /// Measure ticks for the function
    pub fn measure<T>(self) -> (T, u64)
    where
        F: FnOnce() -> T,
    {
        let counter = TickCounter::start();
        let result = (self.f)();
        let ticks = counter.elapsed_ticks();
        (result, ticks)
    }
}

/// Zero-cost abstraction for async tick measurement
#[cfg(feature = "async")]
pub struct AsyncTickMeasurer<F> {
    /// Async function to measure
    f: F,
}

#[cfg(feature = "async")]
impl<F> AsyncTickMeasurer<F> {
    /// Create a new async tick measurer
    pub fn new(f: F) -> Self {
        Self { f }
    }

    /// Measure ticks for the async function
    pub async fn measure<T, Fut>(self) -> (T, u64)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let counter = TickCounter::start();
        let result = (self.f)().await;
        let ticks = counter.elapsed_ticks();
        (result, ticks)
    }
}

/// Performance benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Operation name
    pub operation: String,
    /// Number of iterations
    pub iterations: u64,
    /// Total ticks
    pub total_ticks: u64,
    /// Average ticks per iteration
    pub avg_ticks: f64,
    /// Minimum ticks
    pub min_ticks: u64,
    /// Maximum ticks
    pub max_ticks: u64,
    /// P50 ticks (median)
    pub p50_ticks: u64,
    /// P95 ticks
    pub p95_ticks: u64,
    /// P99 ticks
    pub p99_ticks: u64,
}

impl BenchmarkResult {
    /// Check if benchmark meets hot path budget
    pub fn meets_hot_path_budget(&self) -> bool {
        self.avg_ticks <= HOT_PATH_TICK_BUDGET as f64
    }

    /// Check if P95 meets hot path budget
    pub fn p95_meets_hot_path_budget(&self) -> bool {
        self.p95_ticks <= HOT_PATH_TICK_BUDGET
    }

    /// Format benchmark result as string
    pub fn format(&self) -> String {
        format!(
            "Operation: {}\n  Iterations: {}\n  Avg ticks: {:.2}\n  Min: {} | Max: {} | P50: {} | P95: {} | P99: {}\n  Hot path compliant: {}",
            self.operation,
            self.iterations,
            self.avg_ticks,
            self.min_ticks,
            self.max_ticks,
            self.p50_ticks,
            self.p95_ticks,
            self.p99_ticks,
            if self.meets_hot_path_budget() { "YES" } else { "NO" }
        )
    }
}

/// Benchmark a closure multiple times
///
/// # Example
///
/// ```rust,no_run
/// use chicago_tdd_tools::performance::benchmark;
///
/// let result = benchmark("hot_path_operation", 1000, || {
///     hot_path_operation()
/// });
///
/// assert!(result.meets_hot_path_budget(), "{}", result.format());
/// ```
pub fn benchmark<F, T>(operation: &str, iterations: u64, f: F) -> BenchmarkResult
where
    F: Fn() -> T,
{
    let mut tick_samples = Vec::with_capacity(iterations as usize);

    // Warmup
    for _ in 0..100 {
        let _ = f();
    }

    // Measure
    for _ in 0..iterations {
        let (_, ticks) = measure_ticks(|| f());
        tick_samples.push(ticks);
    }

    // Calculate statistics
    tick_samples.sort();
    let total_ticks: u64 = tick_samples.iter().sum();
    let avg_ticks = total_ticks as f64 / iterations as f64;
    let min_ticks = *tick_samples.first().unwrap_or(&0);
    let max_ticks = *tick_samples.last().unwrap_or(&0);
    let p50_idx = (tick_samples.len() * 50 / 100).saturating_sub(1);
    let p95_idx = (tick_samples.len() * 95 / 100).saturating_sub(1);
    let p99_idx = (tick_samples.len() * 99 / 100).saturating_sub(1);

    BenchmarkResult {
        operation: operation.to_string(),
        iterations,
        total_ticks,
        avg_ticks,
        min_ticks,
        max_ticks,
        p50_ticks: tick_samples.get(p50_idx).copied().unwrap_or(0),
        p95_ticks: tick_samples.get(p95_idx).copied().unwrap_or(0),
        p99_ticks: tick_samples.get(p99_idx).copied().unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_counter_basic() {
        let counter = TickCounter::start();
        std::hint::black_box(42); // Prevent optimization
        let ticks = counter.elapsed_ticks();
        assert!(ticks >= 0);
    }

    #[test]
    fn test_tick_counter_within_budget() {
        let counter = TickCounter::start();
        std::hint::black_box(42);
        // Should pass for any reasonable operation
        assert!(counter.assert_within_budget(1000000).is_ok());
    }

    #[test]
    fn test_measure_ticks() {
        let (result, ticks) = measure_ticks(|| 42);
        assert_eq!(result, 42);
        assert!(ticks >= 0);
    }

    #[test]
    fn test_benchmark() {
        let result = benchmark("test_operation", 100, || std::hint::black_box(42));
        assert_eq!(result.operation, "test_operation");
        assert_eq!(result.iterations, 100);
        assert!(result.avg_ticks >= 0.0);
        assert!(result.min_ticks <= result.max_ticks);
    }

    #[test]
    fn test_tick_measurer() {
        let measurer = TickMeasurer::new(|| 42);
        let (result, ticks) = measurer.measure();
        assert_eq!(result, 42);
        assert!(ticks >= 0);
    }
}
