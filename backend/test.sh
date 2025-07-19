source .env.test

if [ ! -f .env.test ]; then
    echo ".env.test file not found!"
    exit 1
fi

echo "Running database migrations..."
sqlx migrate run --database-url $DATABASE_URL

if [ $? -eq 0 ]; then
    echo "Migrations completed successfully"
else
    echo "Migration failed"
    read -p "Press enter to continue..."
    exit 1
fi

echo "Running tests..."
cargo test
read -p "Press enter to exit..."