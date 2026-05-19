pub struct Discretiser {
    num_states: usize,
    thresholds: Vec<f64>,
    current_state: usize,
    
    // Debouncing state (Section 4.2: Mitigating boundary-straddling vulnerabilities)
    sustained_count: usize,
    debounce_target: usize,
    required_sustain_ticks: usize,
}

impl Discretiser {
    pub fn new(num_states: usize, max_power: f64) -> Self {
        let step = max_power / (num_states as f64);
        let thresholds: Vec<f64> = (1..num_states).map(|i| i as f64 * step).collect();
        
        Self {
            num_states,
            thresholds,
            current_state: 0,
            sustained_count: 0,
            debounce_target: 0,
            required_sustain_ticks: 3, // Requires 3 continuous ticks (0.3s at 10Hz) to transition
        }
    }

    /// Maps continuous power measurement to discrete state with debouncing
    pub fn process(&mut self, power: f64) -> usize {
        let mut target = 0;
        for (i, &t) in self.thresholds.iter().enumerate() {
            if power > t { 
                target = i + 1; 
            }
        }
        
        if target != self.current_state {
            if target == self.debounce_target {
                self.sustained_count += 1;
                if self.sustained_count >= self.required_sustain_ticks {
                    self.current_state = target;
                }
            } else {
                self.debounce_target = target;
                self.sustained_count = 1;
            }
        } else {
            self.sustained_count = 0;
        }
        
        self.current_state
    }
}
