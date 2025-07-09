set -e

echo "Building Kvitter backend and frontend for production..."

echo "Building backend..."
cd backend
cargo build --release
cd ..

echo "Building frontend..."
cd frontend
npm run build
cd ..

echo "Build complete!"

echo "Backend binary located at: backend/target/release/kvitter"
echo "Frontend production files located at: frontend/dist"