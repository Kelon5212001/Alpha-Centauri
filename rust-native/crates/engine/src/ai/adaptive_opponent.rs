#[derive(Debug, Clone, Copy, Default)]
pub struct MoveFeatures {
    pub attack_value: f32,
    pub defense_value: f32,
    pub expansion_value: f32,
    pub risk: f32,
}

#[derive(Debug, Clone)]
pub struct MoveOption {
    pub id: String,
    pub features: MoveFeatures,
}

#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub features: MoveFeatures,
    /// Positive means the move helped, negative means it hurt.
    pub outcome: f32,
}

#[derive(Debug, Clone)]
pub struct AdaptiveOpponent {
    weights: MoveFeatures,
    learning_rate: f32,
}

impl AdaptiveOpponent {
    pub fn new(learning_rate: f32) -> Self {
        Self {
            weights: MoveFeatures {
                attack_value: 0.25,
                defense_value: 0.25,
                expansion_value: 0.25,
                risk: -0.25,
            },
            learning_rate,
        }
    }

    pub fn choose_move<'a>(&self, options: &'a [MoveOption]) -> Option<&'a MoveOption> {
        options.iter().max_by(|a, b| {
            self.score(a.features)
                .partial_cmp(&self.score(b.features))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn learn_from_mistake(&mut self, sample: TrainingSample) {
        let prediction = self.score(sample.features);
        let error = sample.outcome - prediction;
        self.weights.attack_value += self.learning_rate * error * sample.features.attack_value;
        self.weights.defense_value += self.learning_rate * error * sample.features.defense_value;
        self.weights.expansion_value +=
            self.learning_rate * error * sample.features.expansion_value;
        self.weights.risk += self.learning_rate * error * sample.features.risk;
    }

    pub fn weights(&self) -> MoveFeatures {
        self.weights
    }

    fn score(&self, f: MoveFeatures) -> f32 {
        self.weights.attack_value * f.attack_value
            + self.weights.defense_value * f.defense_value
            + self.weights.expansion_value * f.expansion_value
            + self.weights.risk * f.risk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_opponent_updates_weights_after_bad_outcome() {
        let mut ai = AdaptiveOpponent::new(0.2);
        let original = ai.weights();

        ai.learn_from_mistake(TrainingSample {
            features: MoveFeatures {
                attack_value: 1.0,
                defense_value: 0.0,
                expansion_value: 0.0,
                risk: 1.0,
            },
            outcome: -1.0,
        });

        let updated = ai.weights();
        assert!(updated.attack_value < original.attack_value);
        assert!(updated.risk < original.risk);
    }

    #[test]
    fn chooses_highest_scoring_move() {
        let ai = AdaptiveOpponent::new(0.1);
        let options = vec![
            MoveOption {
                id: "safe-expand".to_string(),
                features: MoveFeatures {
                    attack_value: 0.2,
                    defense_value: 0.8,
                    expansion_value: 0.7,
                    risk: 0.1,
                },
            },
            MoveOption {
                id: "risky-attack".to_string(),
                features: MoveFeatures {
                    attack_value: 0.8,
                    defense_value: 0.1,
                    expansion_value: 0.2,
                    risk: 0.9,
                },
            },
        ];

        let chosen = ai.choose_move(&options).expect("at least one option");
        assert_eq!(chosen.id, "safe-expand");
    }
}
