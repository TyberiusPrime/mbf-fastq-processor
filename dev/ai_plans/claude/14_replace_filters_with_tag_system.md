# Step-by-Step Plan: Replace FilterEmpty with Tag System

## 1. Analyze Current Code Structure
- Examine the current FilterEmpty implementation
- Identify ExtractLength, ExtractMeanQuality, ExtractGCContent, ExtractNCount implementations
- Understand TargetPlusAll enum and its current usage

## 2. Extend Extract Functions to Support TargetPlusAll
- Modify ExtractLength to work on TargetPlusAll instead of just specific targets
- Extend ExtractMeanQuality to work on TargetPlusAll
- Extend ExtractGCContent to work on TargetPlusAll  
- Extend ExtractNCount to work on TargetPlusAll
- Create a generic extract_numeric function that can handle all these cases

## 3. Create Test Cases
- Add test cases for ExtractLength with TargetPlusAll
- Add test cases for ExtractMeanQuality with TargetPlusAll
- Add test cases for ExtractGCContent with TargetPlusAll
- Add test cases for ExtractNCount with TargetPlusAll

## 4. Replace FilterEmpty Implementation
- Replace FilterEmpty usage in Transformation::Expand with ExtractLength + FilterByNumericTag combination
- Ensure the new implementation maintains the same filtering behavior as the original FilterEmpty

## 5. Verify and Test
- Run all existing tests to ensure no regressions
- Run the new test cases to verify the extended functionality works correctly
- Update test cases using dev/update_tests.py if needed

