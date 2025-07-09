set -e

echo "Starting Kvitter project setup..."

if ! command -v cargo &> /dev/null
then
    echo "Rust not found. Please install Rust first."
    exit 1
fi

# Check for Node.js and npm
if ! command -v npm &> /dev/null
then
    echo "npm not found. Please install Node.js and npm first."
    exit 1
fi

# Backend setup
echo "📦 Setting up backend..."
cd backend
cargo fetch
cargo build
cd ..

# Frontend setup
echo "📦 Setting up frontend..."
cd frontend
npm install
npm run build
cd ..

echo "Setup complete! You can now run the backend and frontend servers."