use pug_vault::{Vault, VaultData};
use std::collections::HashMap;
use std::time::Instant;

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[test]
    fn benchmark_key_derivation() {
        let password = "benchmark_password_test";
        let start = Instant::now();

        for _ in 0..10 {
            let _ = Vault::derive_key(password).unwrap();
        }

        let duration = start.elapsed();
        println!("Key derivation 10x: {:?}", duration);

        // Should complete in reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 30000); // Less than 30 seconds for 10 iterations
    }

    #[test]
    fn benchmark_encryption_performance() {
        let vault = create_temp_vault("encryption_benchmark");

        // Test with various sizes
        let test_cases = vec![
            ("small", "x".repeat(100)),     // 100 bytes
            ("medium", "y".repeat(1000)),   // 1KB
            ("large", "z".repeat(10000)),   // 10KB
            ("xlarge", "a".repeat(100000)), // 100KB
        ];

        for (size_name, test_data) in test_cases {
            let mut secrets = HashMap::new();
            secrets.insert("test_key".to_string(), test_data);
            let vault_data = VaultData { secrets };

            let start = Instant::now();
            vault.write_data(&vault_data).unwrap();
            let write_duration = start.elapsed();

            let start = Instant::now();
            vault.read_data().unwrap();
            let read_duration = start.elapsed();

            println!(
                "{} write: {:?}, read: {:?}",
                size_name, write_duration, read_duration
            );

            // Performance assertions (adjust thresholds as needed)
            assert!(write_duration.as_millis() < 1000); // Less than 1 second
            assert!(read_duration.as_millis() < 1000); // Less than 1 second
        }
    }

    #[test]
    fn benchmark_multiple_secrets() {
        let vault = create_temp_vault("multiple_secrets_benchmark");

        let mut secrets = HashMap::new();

        // Create 1000 secrets
        let start = Instant::now();
        for i in 0..1000 {
            secrets.insert(format!("key_{}", i), format!("value_{}", i));
        }
        let vault_data = VaultData { secrets };

        vault.write_data(&vault_data).unwrap();
        let write_duration = start.elapsed();

        let start = Instant::now();
        let read_data = vault.read_data().unwrap();
        let read_duration = start.elapsed();

        assert_eq!(read_data.secrets.len(), 1000);

        println!(
            "1000 secrets write: {:?}, read: {:?}",
            write_duration, read_duration
        );

        // Should handle 1000 secrets efficiently
        assert!(write_duration.as_millis() < 5000); // Less than 5 seconds
        assert!(read_duration.as_millis() < 5000); // Less than 5 seconds
    }

    #[test]
    fn stress_test_concurrent_operations() {
        use std::thread;

        let password = "concurrent_test_password";
        let mut handles = vec![];

        let start = Instant::now();

        // Spawn 10 threads that each derive keys
        for i in 0..10 {
            let test_password = format!("{}_thread_{}", password, i);
            let handle = thread::spawn(move || {
                for _ in 0..5 {
                    let _key = Vault::derive_key(&test_password).unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        println!(
            "Concurrent key derivation (10 threads x 5 keys): {:?}",
            duration
        );

        // Should complete in reasonable time
        assert!(duration.as_millis() < 30000); // Less than 30 seconds
    }

    fn create_temp_vault(password: &str) -> Vault {
        use std::env::temp_dir;

        let key = Vault::derive_key(password).unwrap();
        let mut data_file = temp_dir();
        data_file.push(format!("benchmark_vault_{}.data", rand::random::<u32>()));

        Vault { key, data_file }
    }
}

// Memory usage tests
#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_memory_usage_large_vault() {
        let vault = create_temp_vault("memory_test");

        let mut secrets = HashMap::new();

        // Create secrets with incrementally larger values
        for i in 0..100 {
            let size = (i + 1) * 1000; // 1KB, 2KB, 3KB, ... 100KB
            let value = "x".repeat(size);
            secrets.insert(format!("key_{}", i), value);
        }

        let vault_data = VaultData { secrets };

        // This should not cause memory issues
        vault.write_data(&vault_data).unwrap();
        let read_data = vault.read_data().unwrap();

        assert_eq!(read_data.secrets.len(), 100);

        // Verify largest secret is intact
        let largest_secret = read_data.secrets.get("key_99").unwrap();
        assert_eq!(largest_secret.len(), 100000); // 100KB
    }

    fn create_temp_vault(password: &str) -> Vault {
        use std::env::temp_dir;

        let key = Vault::derive_key(password).unwrap();
        let mut data_file = temp_dir();
        data_file.push(format!("memory_test_vault_{}.data", rand::random::<u32>()));

        Vault { key, data_file }
    }
}
