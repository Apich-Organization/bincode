#!/bin/bash

export PROJECT_ROOT_DIR=$(pwd)
export TIMESTAMP=$(date +%Y%m%d_%H%M%S)
export FINAL_OUTPUT_DIR="$PROJECT_ROOT_DIR/high_res_analysis_$TIMESTAMP"

mkdir -p "$FINAL_OUTPUT_DIR"

# --- Function to handle plotting ---
run_plots() {
    local d=$1
    local n=$2
    
    # Check if directory exists before entering
    [ -d "$d" ] || return

    # 1. Growth (Total LOC)
    [ -f "$d/cohorts.json" ] && git-of-theseus-stack-plot "$d/cohorts.json" --outfile "$FINAL_OUTPUT_DIR/${n}_growth.png"
    # 2. Age Mix (%)
    [ -f "$d/cohorts.json" ] && git-of-theseus-stack-plot "$d/cohorts.json" --normalize --outfile "$FINAL_OUTPUT_DIR/${n}_age_mix.png"
    # 3. Authors
    [ -f "$d/authors.json" ] && git-of-theseus-stack-plot "$d/authors.json" --outfile "$FINAL_OUTPUT_DIR/${n}_authors.png"
    # 4. Survival Decay
    [ -f "$d/survival.json" ] && git-of-theseus-survival-plot "$d/survival.json" --outfile "$FINAL_OUTPUT_DIR/${n}_survival.png"
}

# --- Analyze current directory only ---
DATA_DIR="$FINAL_OUTPUT_DIR/ROOT_data"
mkdir -p "$DATA_DIR"

echo "Analyzing current directory..."
git-of-theseus-analyze --outdir "$DATA_DIR" --interval 5400 --all-filetypes "."

# Run Plots
run_plots "$DATA_DIR" "ROOT"

echo "Done! Analysis results are in: $FINAL_OUTPUT_DIR"
