#!/bin/bash
# test_nojs.sh - Test the Hyrule server without JavaScript

echo " Testing Hyrule Server (No JavaScript Required)"
echo "================================================"

BASE_URL="http://localhost:3000"

# Start server in background
echo "Starting server..."
cargo run &
SERVER_PID=$!
sleep 5

echo ""
echo "1⃣ Testing signup (form submission)"
SIGNUP_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/signup" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&password=testpass123" \
  -L)  # Follow redirects

SIGNUP_CODE=$(echo "$SIGNUP_RESPONSE" | tail -n 1)
echo "Signup status code: $SIGNUP_CODE"

if [ "$SIGNUP_CODE" = "200" ]; then
    echo " Signup successful"
else
    echo " Signup failed"
fi

echo ""
echo "2⃣ Testing login (form submission)"
LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/login" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&password=testpass123" \
  -L)

LOGIN_CODE=$(echo "$LOGIN_RESPONSE" | tail -n 1)
echo "Login status code: $LOGIN_CODE"

if [ "$LOGIN_CODE" = "200" ]; then
    echo " Login successful"
else
    echo " Login failed"
fi

echo ""
echo "3⃣ Testing API signup (JSON)"
API_SIGNUP=$(curl -s -X POST "$BASE_URL/api/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{"username":"apiuser","password":"apipass123"}')

TOKEN=$(echo "$API_SIGNUP" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -n "$TOKEN" ]; then
    echo " API signup successful"
    echo "Token: ${TOKEN:0:20}..."
else
    echo " API signup failed"
    echo "$API_SIGNUP"
fi

echo ""
echo "4⃣ Testing repository creation"
if [ -n "$TOKEN" ]; then
    REPO_RESPONSE=$(curl -s -X POST "$BASE_URL/api/repos" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"name":"test-repo","description":"Test repository","storage_tier":"free","is_private":false}')
    
    REPO_HASH=$(echo "$REPO_RESPONSE" | grep -o '"repo_hash":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$REPO_HASH" ]; then
        echo " Repository created"
        echo "Hash: $REPO_HASH"
    else
        echo " Repository creation failed"
        echo "$REPO_RESPONSE"
    fi
fi

echo ""
echo "5⃣ Testing public endpoints"
curl -s "$BASE_URL/api/repos" | head -c 100
echo "..."
echo ""

curl -s "$BASE_URL/api/stats" | grep -o '"[^"]*":[0-9]*' | head -5
echo ""

echo ""
echo "6⃣ Testing health endpoint"
HEALTH=$(curl -s "$BASE_URL/api/health")
echo "$HEALTH" | grep -o '"status":"[^"]*"'
echo ""

echo ""
echo "7⃣ Testing web pages (should work without JavaScript)"
echo "Testing homepage..."
curl -s "$BASE_URL/" | grep -o '<title>[^<]*' | head -1

echo "Testing explore page..."
curl -s "$BASE_URL/explore" | grep -o '<title>[^<]*' | head -1

echo "Testing login page..."
curl -s "$BASE_URL/login" | grep -o '<title>[^<]*' | head -1

echo "Testing signup page..."
curl -s "$BASE_URL/signup" | grep -o '<title>[^<]*' | head -1

echo ""
echo " All tests completed!"
echo ""
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null

echo "Done!"
