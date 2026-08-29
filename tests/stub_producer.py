#!/usr/bin/env python3
"""A producer that finds nothing, for testing the scorer rather than a graph.

It exists so the scorer's own gates can run without building a real
extractor: a corpus whose tests need the tool it grades cannot test the
grading.
"""
import json
import sys

if len(sys.argv) < 4:
    sys.exit("usage: stub_producer.py <org> <repo> <input-dir>")

json.dump({"nodes": [], "edges": [], "mentions": [], "diagnostics": [],
           "stats": {"unresolved_calls": 0}}, sys.stdout)
