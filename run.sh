set -e

echo "Starting Kvitter development servers..."

echo "Starting backend server..."
(cd backend && cargo run) &

echo "Starting frontend dev server..."
(cd frontend && npm run dev) &

wait