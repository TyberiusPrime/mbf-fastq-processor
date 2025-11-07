#!/usr/bin/env bash
# verify that "==== StoreTagInComment" only occurs once in the stderr file
#
ls -la
set -eou pipefail
error=`grep -c "==== StoreTagInComment" stderr`
if [ $error -ne 1 ]; then
	echo "Error: '==== StoreTagInComment' occurred $error times in stderr, not just once"
	exit 1

fi
