# Changelog


## # since v0.8.1 (main)

- Redesigned multi core engine from 'spawn 1 or n threads per step' to 
  workpool based, better controllable workflow, better threading documentation
- Introduced benchmark mode & benchmark test harness
- Rapidgzip for multi-core decompression of gzip files
- Speed improvements for most steps, better (multi-core) parsing
- removed a set of parsing bugs, documented parser
- extended cookbooks 
- lot's of polish.
- nix build now uses nix based rapidgzip if no rapidgzip binary is found.
- histogram tag analysis for reports
- copy to clipboard in docs
- autocompletion for shells
- validation & verify CLI modes to test configuration and verify it produces unchanged output
- better explanation of initial filter capacity for cuckoo filters. 
- Added 'read count estimation' to estimate initial cuckoo filter capacity
- conditional read-editing with if_tag option on steps



## v0.8.1
   
- Github release workflow test


## v0.8.0

- Versioned documentation
- First revision where very major feature is in place. Changelog starts here.

