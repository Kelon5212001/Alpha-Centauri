use super::{try_design_from_content, UnitDesign};

#[derive(Debug, Clone)]
pub enum NativeType {
    MindWorm,
    IsleOfTheDeep,
}

impl NativeType {
    pub fn design(&self) -> UnitDesign {
        self.try_design()
            .expect("bundled native unit content must decode successfully")
    }

    pub fn try_design(&self) -> Result<UnitDesign, super::UnitDesignError> {
        match self {
            NativeType::MindWorm => try_design_from_content("mind_worm"),
            NativeType::IsleOfTheDeep => try_design_from_content("isle_of_the_deep"),
        }
    }
}
