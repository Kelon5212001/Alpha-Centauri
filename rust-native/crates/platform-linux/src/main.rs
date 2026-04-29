use alpha_centauri_engine::ai::{AdaptiveOpponent, MoveFeatures, MoveOption, TrainingSample};

fn main() {
    println!("alpha-centauri linux bootstrap {}", alpha_centauri_engine::version());

    let mut ai = AdaptiveOpponent::new(0.15);
    let options = vec![
        MoveOption {
            id: "push-frontline".to_string(),
            features: MoveFeatures {
                attack_value: 0.7,
                defense_value: 0.3,
                expansion_value: 0.2,
                risk: 0.8,
            },
        },
        MoveOption {
            id: "fortify-and-grow".to_string(),
            features: MoveFeatures {
                attack_value: 0.3,
                defense_value: 0.8,
                expansion_value: 0.8,
                risk: 0.2,
            },
        },
    ];

    if let Some(choice) = ai.choose_move(&options) {
        println!("Initial AI choice: {}", choice.id);
    }

    // Example learning update after a poor result from a risky move.
    ai.learn_from_mistake(TrainingSample {
        features: MoveFeatures {
            attack_value: 0.7,
            defense_value: 0.3,
            expansion_value: 0.2,
            risk: 0.8,
        },
        outcome: -0.9,
    });

    if let Some(choice) = ai.choose_move(&options) {
        println!("Post-learning AI choice: {}", choice.id);
    }
}
