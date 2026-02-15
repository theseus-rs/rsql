#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

root_dir="$(cd "$(dirname "$0")"; pwd)"
echo "root_dir: $root_dir"
cargo build --release
 export PATH="$root_dir/../target/release:$PATH"

# Function to process a single tape file
process_tape_file() {
  local file="$1"
  local script_directory=$(dirname "$file")
  local file_name=$(basename "$file")
  local base_file_name="${file_name%.*}"
  local tape_file="$script_directory/${base_file_name}.tape"
  local webm_file="$script_directory/${base_file_name}.webm"
  local log_file="$script_directory/${base_file_name}.log"

  echo "Processing: $file"

  cd "$script_directory" || { echo "Failed to change directory to $script_directory"; return 1; }

  tape_start=$(cat << EOF
Set FontSize 14
Set Width 800
Set Height 600
Set Padding 20
Type "rsql"
Enter
Sleep 2s
EOF
  )

  tape_end=$(cat << EOF
Enter
Sleep 4s
EOF
  )

  # Clean history and config files before running the demo
  rm -f ~/.rsql/rsql.history
  rm -f ~/.rsql/rsql.toml

  # Run the vhs command and capture its output
  if (echo "$tape_start"; cat "${tape_file}"; echo "$tape_end") | vhs --output "$webm_file" > "$log_file" 2>&1; then
    echo "Successfully created $webm_file"
    return 0
  else
    echo "ERROR: Failed to create $webm_file. See $log_file for details"
    return 1
  fi
}

# Array to hold all background process IDs
pids=()
failures=0

# Find all tape files and process them in parallel
for file in $(find "$root_dir" -name "*.tape"); do
  process_tape_file "$file" &
  pids+=("$!")
  echo "Started process PID $! for file: $file"
done

# Wait for all background processes to finish
echo "Waiting for all processes to complete..."
for pid in "${pids[@]}"; do
  wait "$pid" || {
    echo "Process $pid failed"
    ((failures++))
  }
done

# Report results
echo "All processes completed."
if [ $failures -gt 0 ]; then
  echo "ERROR: $failures process(es) failed"
  exit 1
else
  echo "SUCCESS: All demos created successfully"
fi

