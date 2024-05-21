#!/usr/bin/env bash

root_dir="$(cd "$(dirname "$0")"; pwd)"
echo "root_dir: $root_dir"
cargo build --release
rsql="$root_dir/../target/release/rsql"
clear

for file in $(find $root_dir -name "*.sql"); do
  script_directory=$(dirname "$file")
  file_name=$(basename "$file")
  base_file_name="${file_name%.*}"
  sql_file="$script_directory/${base_file_name}.sql"
  cast_file="$script_directory/${base_file_name}.cast"
  gif_file="$script_directory/${base_file_name}.gif"
  echo "running: $sql_file"

  cd "$script_directory"
  asciinema rec --overwrite --command="$rsql --file $sql_file" "$cast_file"
  agg --rows=30 --cols=100 --renderer=resvg "$cast_file" "$gif_file"
done
