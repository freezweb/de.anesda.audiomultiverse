//! Routing-Matrix
//! 
//! NxM Matrix für vollflexibles Audio-Routing

/// Routing-Matrix für Input->Output Zuordnung
pub struct RoutingMatrix {
    /// Anzahl Eingänge
    pub input_count: usize,
    
    /// Anzahl Ausgänge  
    pub output_count: usize,
    
    /// Matrix [input][output] = gain (0.0 - 1.0)
    pub matrix: Vec<Vec<f32>>,
}

impl RoutingMatrix {
    /// Neue Matrix erstellen
    pub fn new(input_count: usize, output_count: usize) -> Self {
        // Standard-Routing: 1:1 Zuordnung
        let mut matrix = vec![vec![0.0; output_count]; input_count];
        
        // Default: Eingang 0,1 -> Ausgang 0,1 (Stereo)
        // Eingang 2,3 -> Ausgang 0,1 etc.
        for i in 0..input_count {
            if i < output_count {
                matrix[i][i] = 1.0;
            }
        }
        
        Self {
            input_count,
            output_count,
            matrix,
        }
    }

    /// Routing-Punkt setzen
    pub fn set(&mut self, input: usize, output: usize, gain: f32) -> bool {
        if input >= self.input_count || output >= self.output_count {
            return false;
        }
        
        self.matrix[input][output] = gain.clamp(0.0, 1.0);
        true
    }

    /// Routing-Punkt abrufen
    pub fn get(&self, input: usize, output: usize) -> Option<f32> {
        if input >= self.input_count || output >= self.output_count {
            return None;
        }
        
        Some(self.matrix[input][output])
    }

    /// Alle Eingänge für einen Ausgang abrufen (mit Gain > 0)
    pub fn inputs_for_output(&self, output: usize) -> Vec<(usize, f32)> {
        if output >= self.output_count {
            return vec![];
        }
        
        self.matrix
            .iter()
            .enumerate()
            .filter_map(|(input, row)| {
                let gain = row[output];
                if gain > 0.0 {
                    Some((input, gain))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Alle Ausgänge für einen Eingang abrufen (mit Gain > 0)
    pub fn outputs_for_input(&self, input: usize) -> Vec<(usize, f32)> {
        if input >= self.input_count {
            return vec![];
        }
        
        self.matrix[input]
            .iter()
            .enumerate()
            .filter_map(|(output, &gain)| {
                if gain > 0.0 {
                    Some((output, gain))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Matrix zurücksetzen (alle auf 0)
    pub fn clear(&mut self) {
        for row in &mut self.matrix {
            for cell in row {
                *cell = 0.0;
            }
        }
    }

    /// 1:1 Routing setzen
    pub fn set_unity(&mut self) {
        self.clear();
        for i in 0..self.input_count.min(self.output_count) {
            self.matrix[i][i] = 1.0;
        }
    }

    /// Stereo-Link: Zwei Kanäle zusammen routen
    pub fn set_stereo_pair(&mut self, left_in: usize, left_out: usize) {
        let right_in = left_in + 1;
        let right_out = left_out + 1;
        
        if right_in < self.input_count && right_out < self.output_count {
            self.matrix[left_in][left_out] = 1.0;
            self.matrix[right_in][right_out] = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation() {
        let matrix = RoutingMatrix::new(32, 32);
        assert_eq!(matrix.input_count, 32);
        assert_eq!(matrix.output_count, 32);
        
        // Default: 1:1 Routing
        assert_eq!(matrix.get(0, 0), Some(1.0));
        assert_eq!(matrix.get(0, 1), Some(0.0));
    }

    #[test]
    fn test_routing() {
        let mut matrix = RoutingMatrix::new(8, 2);
        
        // Eingang 4 zu Ausgang 0 routen
        assert!(matrix.set(4, 0, 0.5));
        assert_eq!(matrix.get(4, 0), Some(0.5));
        
        // Ungültige Indizes
        assert!(!matrix.set(100, 0, 1.0));
    }

    #[test]
    fn test_inputs_for_output() {
        let mut matrix = RoutingMatrix::new(8, 2);
        matrix.clear();
        
        matrix.set(0, 0, 1.0);
        matrix.set(2, 0, 0.5);
        matrix.set(4, 0, 0.25);
        
        let inputs = matrix.inputs_for_output(0);
        assert_eq!(inputs.len(), 3);
    }
}
