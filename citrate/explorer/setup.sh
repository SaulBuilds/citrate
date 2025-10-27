#!/bin/bash

set -e

echo "🚀 Setting up Citrate Explorer..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from example..."
    cp .env.example .env
    echo "⚠️  Please update .env with your configuration"
fi

# Check for Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    exit 1
fi

echo "📦 Installing dependencies..."
npm install

echo "🗄️ Starting PostgreSQL..."
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker-compose exec -T postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done

echo "✅ PostgreSQL is ready!"

echo "🔧 Running database migrations..."
npx prisma migrate dev --name init

echo "🌱 Generating Prisma client..."
npx prisma generate

echo "📊 Database setup complete!"

# Ask if user wants to start the services
read -p "Do you want to start the explorer now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 Starting explorer services..."
    
    # Start indexer in background
    echo "📈 Starting indexer..."
    npm run indexer:dev &
    INDEXER_PID=$!
    
    # Start Next.js dev server
    echo "🌐 Starting web server..."
    npm run dev &
    NEXT_PID=$!
    
    echo "✨ Citrate Explorer is running!"
    echo "📍 Web interface: http://localhost:3000"
    echo "📍 Database: postgresql://localhost:5432/citrate_explorer"
    echo ""
    echo "Press Ctrl+C to stop all services..."
    
    # Handle Ctrl+C
    trap "echo '🛑 Stopping services...'; kill $INDEXER_PID $NEXT_PID; docker-compose down; exit" INT
    
    # Wait for processes
    wait $INDEXER_PID $NEXT_PID
else
    echo "✅ Setup complete! You can start the explorer with:"
    echo "  npm run dev         # Start web server"
    echo "  npm run indexer     # Start indexer"
    echo ""
    echo "Or use Docker Compose:"
    echo "  docker-compose up"
fi