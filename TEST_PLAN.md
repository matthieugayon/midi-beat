# Integration Test Plan for midi-parse Crate

## Overview
This document outlines the comprehensive integration test design for the `midi-parse` crate. The test will validate the full end-to-end pipeline from MIDI file loading to final dataset generation as an ndarray `ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>`.

## Test Objectives
- Validate the complete MIDI processing pipeline
- Ensure correct shape transformation: 2 bars → 2×32×8×3 array
- Test all major pipeline components in integration
- Verify data integrity throughout the processing chain

## Pipeline Components Analysis

Based on the codebase examination, the full pipeline consists of:

### 1. MIDI File Loading and Parsing
- **Module**: External (`midly` crate)
- **Input**: Raw MIDI file bytes
- **Output**: `midly::Smf` structure
- **Function**: `midly::Smf::parse(&data)`

### 2. Beat Filtering and Event Extraction
- **Module**: `midi_parse::parse`
- **Function**: `filter_beat(smf, drum_channel: bool) -> Vec<DrumTrack>`
- **Process**:
  - Filters tracks with beat events (channel 9 if drum_channel=true)
  - Extracts drum events with timing, velocity, key, and duration
  - Groups events by time signature
  - Merges compatible tracks

### 3. Track Pool Processing and Grid Generation
- **Module**: `midi_parse::map`
- **Function**: `process_track_pool(&Vec<DrumTrack>) -> Result<Array<f32, Ix4>, ShapeError>`
- **Process**:
  - Maps percussion keys to 8 standardized tracks
  - Converts tracks to grid format (bars × steps × tracks × features)
  - Applies filtering for compatible time signatures
  - Removes duplicate bars using quantization

### 4. Statistical Filtering (Optional in Pipeline)
- **Module**: `midi_parse::stats`
- **Functions**: `filter_densities()`, `filter_gridicity()`
- **Note**: These are used in parser-cli but not mandatory for basic pipeline

## Expected Data Flow

```
MIDI File (test_beat.mid)
    ↓ [midly::Smf::parse]
midly::Smf structure
    ↓ [filter_beat]
Vec<DrumTrack>
    ↓ [process_track_pool]
Array<f32, Ix4> with shape [num_bars, 32, 8, 3]
```

## Test Data Specifications

### Input File: `test/test_beat.mid`
- **Expected**: 2-bar MIDI file
- **Time Signature**: 4/4 (expected based on filtering logic)
- **Content**: Drum events on channel 9
- **Resolution**: Compatible with 32-step grid (PPQN divisible by 32)

### Expected Output Shape: `[2, 32, 8, 3]`
- **Dimension 0**: 2 bars
- **Dimension 1**: 32 steps per bar (RESOLUTION constant)
- **Dimension 2**: 8 percussion tracks (NUMBER_OF_TRACKS constant)
- **Dimension 3**: 3 features per event:
  - `[0]`: Normalized velocity (0.0-1.0)
  - `[1]`: Normalized timing offset (-1.0 to 1.0)
  - `[2]`: Normalized duration (0.0-1.0)

## Test Implementation Structure

### Test Function: `test_full_pipeline_integration()`

#### Setup Phase
1. **File Loading**: Read `test/test_beat.mid` from repository
2. **Error Handling**: Ensure file exists and is readable
3. **MIDI Parsing**: Parse with `midly::Smf::parse()`

#### Processing Phase
1. **Beat Filtering**: 
   - Call `filter_beat(smf, true)` for drum channel filtering
   - Validate returned `Vec<DrumTrack>` is non-empty
   
2. **Track Pool Processing**:
   - Call `process_track_pool(&track_pool)`
   - Handle potential `ShapeError`
   - Validate successful array creation

#### Validation Phase
1. **Shape Verification**:
   - Assert array shape equals `[2, 32, 8, 3]`
   - Verify array type is `Array<f32, Ix4>`

2. **Data Integrity Checks**:
   - Verify feature values are within expected ranges:
     - Velocity: 0.0 ≤ val ≤ 1.0
     - Offset: -1.0 ≤ val ≤ 1.0  
     - Duration: 0.0 ≤ val ≤ 1.0
   - Check for reasonable data distribution (not all zeros)

3. **Content Validation**:
   - Verify at least some non-zero events exist
   - Check that events are properly distributed across bars
   - Validate percussion mapping is applied correctly

## Test Dependencies

### Required Crates
- `ndarray` - For array operations and validation
- `midly` - For MIDI parsing (already in dependencies)
- Standard library for file I/O

### Test-specific Dependencies
- `std::fs` - File reading
- `std::path::Path` - Path handling

## Error Scenarios to Test

1. **File Not Found**: Test behavior when MIDI file is missing
2. **Invalid MIDI**: Test behavior with corrupted MIDI data
3. **Empty Track Pool**: Test when no valid tracks are found
4. **Shape Errors**: Test when array creation fails

## Success Criteria

The test passes if:
1. ✅ MIDI file loads successfully
2. ✅ Beat filtering produces valid DrumTracks
3. ✅ Track pool processing completes without errors
4. ✅ Final array has exact shape `[2, 32, 8, 3]`
5. ✅ All feature values are within valid ranges
6. ✅ Array contains meaningful drum data (not all zeros)

## Implementation Location

- **File**: `midi-parse/src/lib.rs` (add test module)
- **Alternative**: `midi-parse/tests/integration_test.rs` (separate integration test file)
- **Test Data**: Reference `../test/test_beat.mid` from test location

## Additional Considerations

### Performance Validation
- Measure processing time for 2-bar file
- Ensure reasonable memory usage

### Cross-validation with Parser-CLI
- Compare results with parser-cli output on same file
- Verify consistency in processing logic

### Future Extensions
- Test with different time signatures
- Test with larger MIDI files
- Test with non-drum channel data