We want to replace FilterEmpty with a combination of 
ExtracLength and FilterByNumericTag.

To do this, we will need to extend ExtractLength to work on
TargetPlusAll. While we're at this, once we have an extended extract_numeric
function,  we can also do this for ExtractMeanQuality, ExtractGCContent,
ExtractNCount. Add test cases for this.

Once that's done, we'll replace FilterEmpty  in Transformation::Expand

