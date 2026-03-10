use sysinfo::System;

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub per_core: Vec<f32>,
    pub overall: f32,
}

#[derive(Debug, Clone)]
pub struct MemSample {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub process_rss_bytes: u64,
}

pub struct MetricsSampler {
    sys: System,
}

impl MetricsSampler {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn cpu_sample(&self) -> CpuSample {
        let cpus = self.sys.cpus();
        let per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let overall = if per_core.is_empty() {
            0.0
        } else {
            per_core.iter().sum::<f32>() / per_core.len() as f32
        };
        CpuSample { per_core, overall }
    }

    pub fn mem_sample(&self) -> MemSample {
        let total_bytes = self.sys.total_memory();
        let used_bytes = self.sys.used_memory();
        let process_rss_bytes = sysinfo::get_current_pid()
            .ok()
            .and_then(|pid| self.sys.process(pid))
            .map(|p| p.memory())
            .unwrap_or(0);
        MemSample {
            total_bytes,
            used_bytes,
            process_rss_bytes,
        }
    }
}

impl Default for MetricsSampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_refresh() {
        let mut sampler = MetricsSampler::new();
        sampler.refresh();
        // Just verify no panic
    }

    #[test]
    fn test_cpu_sample_per_core_nonempty() {
        let mut sampler = MetricsSampler::new();
        sampler.refresh();
        let sample = sampler.cpu_sample();
        assert!(
            !sample.per_core.is_empty(),
            "Should have at least 1 CPU core"
        );
    }

    #[test]
    fn test_cpu_sample_overall_in_range() {
        let mut sampler = MetricsSampler::new();
        sampler.refresh();
        let sample = sampler.cpu_sample();
        assert!(sample.overall >= 0.0);
        assert!(sample.overall <= 100.0);
    }

    #[test]
    fn test_mem_sample_total_nonzero() {
        let mut sampler = MetricsSampler::new();
        sampler.refresh();
        let sample = sampler.mem_sample();
        assert!(sample.total_bytes > 0, "Total memory should be > 0");
    }

    #[test]
    fn test_mem_sample_used_lte_total() {
        let mut sampler = MetricsSampler::new();
        sampler.refresh();
        let sample = sampler.mem_sample();
        assert!(sample.used_bytes <= sample.total_bytes);
    }

    #[test]
    fn test_default_same_as_new() {
        let _sampler = MetricsSampler::default();
        // Just verifies Default trait is implemented
    }
}
