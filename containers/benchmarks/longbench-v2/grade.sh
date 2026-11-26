#!/bin/bash
# Verifier: upstream's answer extraction on the agent's stdout (grade.py).
mkdir -p /logs/verifier
echo 0 > /logs/verifier/reward.txt   # fail-closed (rule 18)
python3 /tests/grade.py
