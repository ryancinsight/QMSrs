use anyhow::Result;
use qmsrs::config::Config;

fn main() -> Result<()> {
    // Initialize the QMS system
    println!("QMSrs - FDA Compliant Medical Device Quality Management System");
    println!("Version: {}", qmsrs::APPLICATION_VERSION);
    println!("FDA CFR Part 820 Version: {}", qmsrs::FDA_CFR_PART_820_VERSION);
    println!("ISO 13485 Version: {}", qmsrs::ISO_13485_VERSION);
    
    // Load default configuration
    let config = Config::default();
    
    // Validate FDA compliance
    config.validate()?;
    
    println!("\n✓ FDA compliance validation passed");
    println!("✓ Organization: {}", config.application.organization_name);
    println!("✓ Audit retention: {} days", config.compliance.audit_retention_days);
    println!("✓ CFR Part 11 mode: {}", config.compliance.cfr_part_11_mode);
    println!("✓ Electronic signatures: {}", config.compliance.require_electronic_signatures);
    
    // Generate sample configuration
    println!("\nGenerating sample configuration...");
    let sample_config = Config::generate_sample();
    println!("Sample configuration:\n{}", sample_config);
    
    println!("\n✓ QMS system initialized successfully");
    println!("Ready for FDA-compliant medical device quality management");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_application_startup() {
        // Test that the main function runs without errors
        let result = main();
        assert!(result.is_ok(), "Main application should start successfully");
    }
}