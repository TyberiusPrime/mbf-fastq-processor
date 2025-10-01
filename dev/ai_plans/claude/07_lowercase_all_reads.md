# outcome: success
# Update LowercaseSequence and UppercaseSequence to support TargetPlusAll

## Completed Tasks

✅ **Research current implementations**: Found that `LowercaseSequence` and `UppercaseSequence` were incorrectly using `Target` instead of `TargetPlusAll`

✅ **Update LowercaseSequence struct**: Changed `target: Target` to `target: TargetPlusAll` to support the `All` option

✅ **Update UppercaseSequence struct**: Changed `target: Target` to `target: TargetPlusAll` to support the `All` option  

✅ **Fix validation functions**: Updated both transformations to use `validate_target_plus_all` instead of `validate_target`

✅ **Update apply functions**: Replaced `apply_in_place_wrapped` with `apply_in_place_wrapped_plus_all` for both transformations

✅ **Add required imports**: Added `apply_in_place_wrapped_plus_all`, `validate_target_plus_all`, and `TargetPlusAll` to the imports

✅ **Test changes**: All tests pass, confirming the transformations work correctly with the new TargetPlusAll support

## Summary

Both `LowercaseSequence` and `UppercaseSequence` transformations now correctly accept `TargetPlusAll` instead of just `Target`, allowing users to specify `target = "all"` to apply the transformation to all available reads (read1, read2, index1, index2). The implementation uses `apply_in_place_wrapped_plus_all` which handles the `All` case by applying the transformation to each available target.
