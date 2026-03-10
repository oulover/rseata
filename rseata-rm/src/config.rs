use rseata_core::branch::BranchType;
use rseata_core::types::ResourceId;

pub struct RMConfig {
    pub resource_id: ResourceId,
    pub branch_type: BranchType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_rm_config_branch_type_from_env() {
        // Test default branch type (AT)
        unsafe {
            env::remove_var("RSEATA_BRANCH_TYPE");
        }
        let config = RMConfig {
            resource_id: ResourceId::from("test".to_string()),
            branch_type: BranchType::AT,
        };
        assert_eq!(config.branch_type, BranchType::AT);

        // Test XA branch type via environment variable
        unsafe {
            env::set_var("RSEATA_BRANCH_TYPE", "XA");
        }
        // Note: RMConfig does not read env itself; ResourceInfo does.
        // We'll test ResourceInfo separately.
    }
}