#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub used: bool,
    pub alpha: f32,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
            used: false,
            alpha: 0.0,
        }
    }

    pub fn low_pass_filter(self, sample: f32) -> f32 {
        let alpha = self.alpha / 1000.0; // 滤波器系数
        static mut LAST_SAMPLE: f32 = 0.0;
        unsafe {
            LAST_SAMPLE = alpha * sample + (1.0 - alpha) * LAST_SAMPLE;
            LAST_SAMPLE
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
