# make usre 'Usage' is in file 'stdout'
STDOUT_CONTENT=$(cat stdout)
echo $STDOUT_CONTENT
if [[ $STDOUT_CONTENT != *"Usage"* ]]; then
  echo "Error: 'Usage' not found in stdout"
  exit 1
fi

