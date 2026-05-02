use std::time::Instant;

#[test]
fn test_sleep_precision_analysis() {
    println!("\n=== Sleep Precision Analysis ===");

    // Test different sleep durations
    let test_cases = vec![1, 5, 10, 50, 100];

    for ms in test_cases {
        let iterations = 10;
        let mut total_overhead = 0u128;

        for _ in 0..iterations {
            let start = Instant::now();
            std::thread::sleep(std::time::Duration::from_millis(ms));
            let elapsed = start.elapsed().as_millis();

            // Overhead = actual time - expected time
            let overhead = if elapsed > ms as u128 {
                elapsed - ms as u128
            } else {
                0
            };
            total_overhead += overhead;
        }

        let avg_overhead = total_overhead / iterations as u128;
        println!(
            "sleep {}ms: avg overhead = {}ms ({}% overhead)",
            ms,
            avg_overhead,
            (avg_overhead * 100 / ms as u128)
        );
    }

    println!("\nKey findings:");
    println!("- Windows thread sleep has ~1-2ms minimum granularity");
    println!("- Short sleeps (1-5ms) have high relative overhead");
    println!("- This explains mixed instruction performance degradation\n");
}
