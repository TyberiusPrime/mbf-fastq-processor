status: open
# investigate illumiana tile information
- can we extract it, report it, plot on it?


https://support.illumina.com/help/BaseSpace_Sequence_Hub_OLH_009008_2/Source/Informatics/BS/FileFormat_FASTQ-files_swBS.htm
ach entry in a FASTQ file consists of four lines:
	• 	Sequence identifier
	• 	Sequence
	• 	Quality score identifier line (consisting only of a +)
	• 	Quality score
The first line, identifying the sequence, contains the following elements.
@<instrument>:<run number>:<flowcell ID>:<lane>:<tile>:<x-pos>:<y-pos>:<UMI> <read>:<is filtered>:<control number>:<index>
FASTQ File Elements
Element
	
Requirements
	
Description
@
	
@
	
Each sequence identifier line starts with @.
<instrument>
	
Characters allowed:
a–z, A–Z, 0–9 and underscore
	
Instrument ID.
<run number>
	
Numerical
	
Run number on instrument.
<flowcell ID>
	
Characters allowed:
a–z, A–Z, 0–9
	
Flow cell ID
<lane>
	
Numerical
	
Lane number.
<tile>
	
Numerical
	
Tile number.
<x_pos>
	
Numerical
	
X coordinate of cluster.
<y_pos>
	
Numerical
	
Y coordinate of cluster.
<UMI>
	
Restricted characters: A/T/G/C/N
	
Optional, appears when UMI is specified in the sample sheet. UMI sequences for Read 1 and Read 2, separated by a plus [+].
<read>
	
Numerical
	
Read number. 1 can be single read or Read 2 of paired-end.
<is filtered>
	
Y or N
	
Y if the read is filtered (did not pass), N otherwise.
<control number>
	
Numerical
	
0 when none of the control bits are on, otherwise it is an even number.
For systems that do not perform control specification, this number is always 0.
<index>
	
Restricted characters: A/T/G/C/N
	
Index of the read.
An example of a valid entry is as follows; note the space preceding the read number element:
@SIM:1:FCX:1:15:6329:1045:GATTACT+GTCTTAAC 1:N:0:ATCCGA
TCGCACTCAACGCCCTGCATATGACAAGACAGAATC
+
<>;##=><9=AAAAAAAAAA9#:<#<;<<<????#=
