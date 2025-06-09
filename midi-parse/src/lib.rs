pub mod datatypes;
pub mod map;
pub mod parse;
pub mod stats;
pub mod utils;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use ndarray::{Array, Ix4};
    use midly::Smf;
    use crate::parse::filter_beat;
    use crate::map::process_track_pool;

    #[test]
    fn test_full_pipeline_integration() {
        // Test file path - relative to the crate root when running tests
        let test_file_path = "../test/test_beat.mid";
        
        // Phase 1: Load and parse MIDI file
        let midi_data = fs::read(test_file_path)
            .expect("Failed to read test MIDI file. Ensure test/test_beat.mid exists in repository root.");
        
        let smf = Smf::parse(&midi_data)
            .expect("Failed to parse MIDI file. The test file may be corrupted.");
        
        println!("✅ MIDI file loaded and parsed successfully");
        
        // Phase 2: Beat filtering and track extraction
        let track_pool = filter_beat(smf, false); // false = don't filter on drum channel, accept all channels
        
        assert!(!track_pool.is_empty(), 
            "No tracks found in MIDI file. Expected at least one track with musical events.");
        
        println!("✅ Beat filtering completed. Found {} drum tracks", track_pool.len());
        
        // Phase 3: Process track pool into ndarray
        let result_array = process_track_pool(&track_pool)
            .expect("Failed to process track pool into array. Check time signature compatibility and track mapping.");
        
        println!("✅ Track pool processing completed");
        
        // Phase 4: Validate expected shape [2, 32, 8, 3]
        let expected_shape = [2, 32, 8, 3];
        let actual_shape = result_array.shape();
        
        assert_eq!(actual_shape, expected_shape,
            "Array shape mismatch. Expected {:?}, got {:?}", expected_shape, actual_shape);
        
        println!("✅ Array shape validation passed: {:?}", actual_shape);
        
        // Phase 5: Data integrity checks
        validate_array_data_integrity(&result_array);
        
        println!("✅ Full pipeline integration test completed successfully!");
        println!("Final array shape: {:?}", result_array.shape());
        
        // Additional debug information
        let non_zero_elements = result_array.iter().filter(|&&x| x != 0.0).count();
        let total_elements = result_array.len();
        let density = non_zero_elements as f32 / total_elements as f32;
        
        println!("Array density: {:.2}% ({}/{} non-zero elements)", 
            density * 100.0, non_zero_elements, total_elements);
    }
    
    fn validate_array_data_integrity(array: &Array<f32, Ix4>) {
        let mut velocity_violations = 0;
        let mut offset_violations = 0;
        let mut duration_violations = 0;
        let mut non_zero_events = 0;
        
        // Iterate through all events in the array
        for bar_idx in 0..array.shape()[0] {
            for step_idx in 0..array.shape()[1] {
                for track_idx in 0..array.shape()[2] {
                    let velocity = array[[bar_idx, step_idx, track_idx, 0]];
                    let offset = array[[bar_idx, step_idx, track_idx, 1]];
                    let duration = array[[bar_idx, step_idx, track_idx, 2]];
                    
                    // Count non-zero events (any event with velocity > 0)
                    if velocity > 0.0 {
                        non_zero_events += 1;
                    }
                    
                    // Validate velocity range [0.0, 1.0]
                    if velocity < 0.0 || velocity > 1.0 {
                        velocity_violations += 1;
                    }
                    
                    // Validate offset range [-1.0, 1.0]
                    if offset < -1.0 || offset > 1.0 {
                        offset_violations += 1;
                    }
                    
                    // Validate duration range [0.0, 1.0]
                    if duration < 0.0 || duration > 1.0 {
                        duration_violations += 1;
                    }
                }
            }
        }
        
        // Assert no violations
        assert_eq!(velocity_violations, 0, 
            "Found {} velocity values outside range [0.0, 1.0]", velocity_violations);
        assert_eq!(offset_violations, 0, 
            "Found {} offset values outside range [-1.0, 1.0]", offset_violations);
        assert_eq!(duration_violations, 0, 
            "Found {} duration values outside range [0.0, 1.0]", duration_violations);
        
        // Assert we have meaningful data
        assert!(non_zero_events > 0, 
            "Array contains no drum events (all velocities are 0). Expected some drum data.");
        
        println!("✅ Data integrity validation passed");
        println!("   - Found {} drum events", non_zero_events);
        println!("   - All velocities in range [0.0, 1.0]");
        println!("   - All offsets in range [-1.0, 1.0]");
        println!("   - All durations in range [0.0, 1.0]");
    }
    
    #[test]
    fn test_pipeline_error_handling() {
        // Test with non-existent file
        let result = fs::read("non_existent_file.mid");
        assert!(result.is_err(), "Expected error when reading non-existent file");
        
        // Test with invalid MIDI data
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid MIDI header
        let parse_result = Smf::parse(&invalid_data);
        assert!(parse_result.is_err(), "Expected error when parsing invalid MIDI data");
        
        println!("✅ Error handling tests passed");
    }
}