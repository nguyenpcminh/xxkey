use vietime_engine::datatype::InputType;

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ToggleEnabled,
    SetInputType(InputType),
    OpenSettings,
    Exit,
}

pub struct TrayMenuConfig {
    pub enabled: bool,
    pub current_input_type: InputType,
}

impl Default for TrayMenuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            current_input_type: InputType::Telex,
        }
    }
}

/// Helper function to format input method status for display
pub fn input_type_label(it: InputType) -> &'static str {
    match it {
        InputType::Telex => "Telex",
        InputType::Vni => "VNI",
        InputType::SimpleTelex1 => "Simple Telex 1",
        InputType::SimpleTelex2 => "Simple Telex 2",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_labels() {
        assert_eq!(input_type_label(InputType::Telex), "Telex");
        assert_eq!(input_type_label(InputType::Vni), "VNI");
        assert_eq!(input_type_label(InputType::SimpleTelex1), "Simple Telex 1");
        assert_eq!(input_type_label(InputType::SimpleTelex2), "Simple Telex 2");
    }
}
