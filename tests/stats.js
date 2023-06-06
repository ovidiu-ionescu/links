#!/usr/bin/bash

curl -v "http://localhost:3000/link_stats" \
  -H "Content-Type: application/json" \
  -H "X-SSL-Client-Verify: SUCCESS" \
  -H "X-SSL-Client-S-DN: CN=ovidiu"

echo ""
