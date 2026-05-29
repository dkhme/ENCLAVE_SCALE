pub struct Discretiser {
    num_states: usize,
    thresholds: Vec<f64>,
}

impl Discretiser {
    pub fn new(hardware: &str) -> Self {
        let thresholds = match hardware {
            "H100" => vec![100.0, 280.0, 460.0, 600.0, 700.0],
            "A100" => vec![60.0, 150.0, 250.0, 350.0, 400.0],
            "L4" => vec![12.0, 28.0, 48.0, 62.0, 72.0],
            _ => panic!("Unsupported hardware type"),
        };
        
        Self {
            num_states: thresholds.len(),
            thresholds,
        }
    }

    /// Maps continuous power measurement to discrete state memoryless mapping
    pub fn process(&self, power: f64) -> usize {
        for (i, &t) in self.thresholds.iter().enumerate() {
            if power < t { 
                return i; 
            }
        }
        self.num_states - 1
    }
}
