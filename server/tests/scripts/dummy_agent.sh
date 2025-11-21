#!/bin/bash
# dummy_agent.sh - Simulates a Vibe Agent

echo "[DummyAgent] Starting up in $(pwd)"

# 1. Read Instruction
if [ -f "INSTRUCTION.md" ]; then
    INSTR=$(cat INSTRUCTION.md)
    echo "[DummyAgent] Read instruction: $INSTR"
else
    echo "[DummyAgent] No INSTRUCTION.md found!"
    exit 1
fi

# 2. Simulate Work (Sleep)
echo "[DummyAgent] Thinking..."
sleep 2

# 3. Write Result
echo "# Result" > RESULT.md
echo "Processed instruction: $INSTR" >> RESULT.md
echo "Agent completed successfully." >> RESULT.md

echo "[DummyAgent] Finished."
exit 0
