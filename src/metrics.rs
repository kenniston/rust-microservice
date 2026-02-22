//! # Metrics Module
//!
//! This module provides the necessary components to collect and
//! expose system and process metrics to the Prometheus server.
//!
//! The metrics collected by this module are:
//!
//! - CPU usage percentage
//! - Memory usage percentage
//! - Number of open file descriptors
//! - Number of open network connections
//!
//! The metrics are exposed as Prometheus counters and can be scraped
//! by the Prometheus server.
//!
//! The module provides a single function, `register_metrics`, which
//! registers all the necessary metrics with the Prometheus registry.
//!
//! The metrics are collected using the `sysinfo` crate, which provides
//! a safe and easy-to-use API for collecting system metrics.
//!
//! The metrics are exposed using the `prometheus` crate, which provides
//! a safe and easy-to-use API for exposing metrics to the Prometheus server.

use prometheus::{
    IntGauge, IntGaugeVec, Opts,
    core::{Collector, Desc},
    process_collector::pid_t,
    proto,
};
use std::thread;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

/// Number of metrics exposed by this collector.
const METRICS_NUMBER: usize = 8;

/// A Prometheus collector that exposes system CPU information.
///
/// `SysInfoCollector` collects CPU-related metrics for a given process ID
/// and publishes them under a configurable namespace.
///
/// ## Exposed Metrics
///
/// - `process_total_cpu_usage`  
///   Total CPU usage expressed as a percentage.  
///   On multi-core systems, this value may exceed 100%. To normalize the
///   value to the range `0–100%`, divide it by the number of CPUs.
///
/// - `process_system_cpu_count`  
///   Total number of CPUs detected on the system.
pub(crate) struct SysInfoCollector {
    /// Process identifier associated with this collector.
    _pid: pid_t,
    /// Metric descriptors required by the Prometheus `Collector` trait.
    descs: Vec<Desc>,
    /// Counter representing the total CPU usage.
    total_cpu_usage: IntGauge,
    /// A counter that represents CPU usage per core.
    cpu_usage: IntGaugeVec,
    /// Counter representing the number of CPUs.
    cpu_count: IntGauge,
    // Counter that represents the Network received bytes.
    network_received: IntGaugeVec,
    // Counter that represents the Network transmitted bytes.
    network_transmitted: IntGaugeVec,
    // Couter that represents the System memory usage.
    system_memory_usage: IntGauge,
    // Couter that represents the System total available memory.
    system_total_memory: IntGauge,
}

impl SysInfoCollector {
    /// Creates a new `SysInfoCollector` for the given process ID.
    ///
    /// # Arguments
    ///
    /// `pid` - The process ID to associate with this collector.
    /// `namespace` - The Prometheus namespace under which the metrics
    ///   will be exposed.
    ///
    /// # Returns
    ///
    /// A fully initialized `SysInfoCollector` with all metric descriptors
    /// registered.
    pub fn with_process_and_namespace<S: Into<String>>(
        _pid: pid_t,
        namespace: S,
    ) -> Result<SysInfoCollector, prometheus::Error> {
        let namespace = namespace.into();
        let mut descs = Vec::new();

        // Helper para Opts com namespace
        let opts = |name: &str, help: &str| Opts::new(name, help).namespace(namespace.clone());

        // Helper para registrar descrições
        let mut collect_descs = |collector: &dyn prometheus::core::Collector| {
            descs.extend(collector.desc().into_iter().cloned());
        };

        // Total CPU usage
        let total_cpu_usage = IntGauge::with_opts(opts(
            "process_total_cpu_usage",
            "Total CPU utilization (in %). Divide by CPU count to get a value between 0% and 100%.",
        ))?;
        collect_descs(&total_cpu_usage);

        // CPU count
        let cpu_count = IntGauge::with_opts(opts(
            "process_system_cpu_count",
            "Returns the list of the CPUs.",
        ))?;
        collect_descs(&cpu_count);

        // CPU usage per core
        let cpu_usage = IntGaugeVec::new(
            opts(
                "process_system_cpu_usage_per_core",
                "CPU utilization (in %) per core.",
            ),
            &["core"],
        )?;
        collect_descs(&cpu_usage);

        // Network received bytes
        let network_received = IntGaugeVec::new(
            opts(
                "process_system_network_received_bytes",
                "Network received bytes.",
            ),
            &["interface"],
        )?;
        collect_descs(&network_received);

        // Network transmitted bytes
        let network_transmitted = IntGaugeVec::new(
            opts(
                "process_system_network_transmitted_bytes",
                "Network transmitted bytes.",
            ),
            &["interface"],
        )?;
        collect_descs(&network_transmitted);

        // System memory usage
        let system_memory_usage = IntGauge::with_opts(opts(
            "process_system_memory_usage",
            "System memory utilization in bytes.",
        ))?;
        collect_descs(&system_memory_usage);

        // System total memory
        let system_total_memory = IntGauge::with_opts(opts(
            "process_system_total_memory",
            "System total memory in bytes.",
        ))?;
        collect_descs(&system_total_memory);

        Ok(SysInfoCollector {
            _pid,
            descs,
            total_cpu_usage,
            cpu_usage,
            cpu_count,
            network_received,
            network_transmitted,
            system_memory_usage,
            system_total_memory,
        })
    }

    /// Creates a `SysInfoCollector` for the current running process.
    ///
    /// # Arguments
    ///
    /// `namespace` - The Prometheus namespace under which the metrics
    ///   will be exposed.
    ///
    /// # Returns
    ///
    /// A `SysInfoCollector` initialized with the PID of the calling process.
    pub fn _with_namespace<S: Into<String>>(
        namespace: S,
    ) -> Result<SysInfoCollector, prometheus::Error> {
        let pid = std::process::id();
        SysInfoCollector::with_process_and_namespace(pid as i32, namespace)
    }
}

/// Implementation of the `Collector` trait for the `SysInfoCollector`.
impl Collector for SysInfoCollector {
    /// Returns the metric descriptors exposed by this collector.
    fn desc(&self) -> Vec<&Desc> {
        self.descs.iter().collect()
    }

    /// Collects the current metric values and returns them as
    /// Prometheus `MetricFamily` instances.
    ///
    /// This method refreshes CPU information using `sysinfo`, updates
    /// the internal counters, and exports the resulting metrics.
    fn collect(&self) -> Vec<proto::MetricFamily> {
        let mut networks = Networks::new_with_refreshed_list();

        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        // CPU usage is a differential value (based on the difference between two measurements).
        // The first call provides the baseline.
        sys.refresh_cpu_all();

        // Wait for the minimum required interval to get an accurate reading.
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        // Refresh CPUs again to get the actual usage value.
        sys.refresh_cpu_all();

        // CPU Count
        self.cpu_count.set(sys.cpus().len() as i64);

        // Total CPU usage
        self.total_cpu_usage.set(sys.global_cpu_usage() as i64);

        // Per-core CPU usage
        sys.cpus().iter().for_each(|cpu| {
            let number = cpu
                .name()
                .strip_prefix("cpu")
                .expect("Invalid CPU index in metrics.");
            let core_index: u32 = number.parse().unwrap_or(0);
            let core_label = format!("CPU{:02}", core_index + 1);
            let gauge = self.cpu_usage.with_label_values(&[&core_label]);
            gauge.set(cpu.cpu_usage() as i64);
        });

        // Network interfaces name, total data received and total data transmitted
        networks.refresh(false);
        for (interface_name, data) in &networks {
            let received_gauge = self.network_received.with_label_values(&[&interface_name]);
            received_gauge.set(data.received() as i64);

            let transmitted_gauge = self
                .network_transmitted
                .with_label_values(&[&interface_name]);
            transmitted_gauge.set(data.transmitted() as i64);
        }

        // System memory
        sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        self.system_memory_usage.set(sys.used_memory() as i64);
        self.system_total_memory.set(sys.total_memory() as i64);

        // collect MetricFamilys.
        let mut mfs = Vec::with_capacity(METRICS_NUMBER);
        mfs.extend(self.total_cpu_usage.collect());
        mfs.extend(self.cpu_usage.collect());
        mfs.extend(self.cpu_count.collect());
        mfs.extend(self.network_received.collect());
        mfs.extend(self.network_transmitted.collect());
        mfs.extend(self.system_memory_usage.collect());
        mfs.extend(self.system_total_memory.collect());
        mfs
    }
}
