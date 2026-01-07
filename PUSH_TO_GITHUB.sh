#!/bin/bash

echo "=================================="
echo "GitHub Deployment Script"
echo "=================================="
echo ""

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "❌ Error: git is not installed"
    exit 1
fi

echo "✓ Git is installed"
echo ""

# Step 1: Check current directory
echo "📁 Current directory: $(pwd)"
echo ""

# Step 2: Initialize git repository
echo "🔧 Step 1: Initializing git repository..."
if [ -d .git ]; then
    echo "⚠️  Git repository already exists"
else
    git init
    echo "✓ Git repository initialized"
fi
echo ""

# Step 3: Configure git (if needed)
echo "🔧 Step 2: Checking git configuration..."
if [ -z "$(git config user.name)" ]; then
    echo "❌ Git user.name not set"
    echo "Run: git config --global user.name 'Your Name'"
    exit 1
fi

if [ -z "$(git config user.email)" ]; then
    echo "❌ Git user.email not set"
    echo "Run: git config --global user.email 'your.email@example.com'"
    exit 1
fi

echo "✓ Git user: $(git config user.name) <$(git config user.email)>"
echo ""

# Step 4: Show what will be committed
echo "🔧 Step 3: Files to be committed..."
git add -n . | head -20
echo "... (showing first 20 files)"
echo ""

# Step 5: Ask for GitHub repository URL
echo "🔧 Step 4: GitHub Repository Setup"
echo ""
echo "IMPORTANT: Before continuing, you need to:"
echo "1. Go to https://github.com/new"
echo "2. Create a new repository named: salesforce-client"
echo "3. Make it PUBLIC"
echo "4. DO NOT initialize with README, .gitignore, or license"
echo "5. Copy the repository URL (e.g., https://github.com/yourusername/salesforce-client.git)"
echo ""
read -p "Have you created the GitHub repository? (yes/no): " CREATED_REPO

if [ "$CREATED_REPO" != "yes" ]; then
    echo "❌ Please create the repository first, then run this script again"
    exit 1
fi

echo ""
read -p "Enter your GitHub repository URL: " REPO_URL

if [ -z "$REPO_URL" ]; then
    echo "❌ Repository URL cannot be empty"
    exit 1
fi

echo "✓ Repository URL: $REPO_URL"
echo ""

# Step 6: Add all files
echo "🔧 Step 5: Adding all files to git..."
git add .
echo "✓ Files added"
echo ""

# Step 7: Create commit
echo "🔧 Step 6: Creating initial commit..."
git commit -m "feat: initial release v0.2.0 - Enterprise Salesforce API client

- OAuth 2.0 with automatic token refresh
- Intelligent caching with TTL/TTI
- Retry logic with exponential backoff
- Rate limiting to respect API quotas
- Automatic pagination for large datasets
- Type-safe query builder
- Full CRUD operations (Create, Update, Delete, Upsert)
- Comprehensive error handling (10 error types)
- Zero unsafe code
- 9 production modules
- 5 comprehensive examples
- Professional documentation"

echo "✓ Initial commit created"
echo ""

# Step 8: Add remote
echo "🔧 Step 7: Adding GitHub remote..."
git remote remove origin 2>/dev/null
git remote add origin "$REPO_URL"
echo "✓ Remote added"
echo ""

# Step 9: Rename branch to main
echo "🔧 Step 8: Renaming branch to main..."
git branch -M main
echo "✓ Branch renamed"
echo ""

# Step 10: Push to GitHub
echo "🔧 Step 9: Pushing to GitHub..."
echo "This may ask for your GitHub credentials..."
echo ""

if git push -u origin main; then
    echo ""
    echo "=================================="
    echo "✅ SUCCESS!"
    echo "=================================="
    echo ""
    echo "Your code has been pushed to GitHub!"
    echo ""
    echo "Repository: $REPO_URL"
    echo ""
    echo "Next steps:"
    echo "1. Visit your repository on GitHub"
    echo "2. Add topics: rust, salesforce, api-client, oauth, async"
    echo "3. Enable GitHub Actions"
    echo "4. Create a release (optional):"
    echo "   git tag -a v0.2.0 -m 'Release v0.2.0'"
    echo "   git push origin v0.2.0"
    echo ""
else
    echo ""
    echo "=================================="
    echo "❌ PUSH FAILED"
    echo "=================================="
    echo ""
    echo "Common issues:"
    echo "1. Authentication failed - you may need to use a Personal Access Token"
    echo "2. Repository doesn't exist - verify you created it on GitHub"
    echo "3. No permission - check you own the repository"
    echo ""
    echo "For authentication issues, see:"
    echo "https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens"
    exit 1
fi
