# Manual GitHub Push Instructions

If you prefer to push manually, follow these exact steps:

## Prerequisites

1. Have a GitHub account
2. Git installed on your system
3. Git configured with your name and email

Check git configuration:
```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## Step-by-Step Process

### 1. Create GitHub Repository

1. Go to: https://github.com/new
2. Fill in:
   - **Repository name:** `salesforce-client`
   - **Description:** `Production-grade Salesforce REST API client for Rust`
   - **Visibility:** Public
   - **DO NOT** check any boxes (no README, no .gitignore, no license)
3. Click "Create repository"
4. **Copy the repository URL** shown (e.g., `https://github.com/yourusername/salesforce-client.git`)

### 2. Initialize Git in Your Project

```bash
# Make sure you're in the project directory
cd /path/to/your/salesforce-client

# Initialize git
git init
```

### 3. Configure Git User (if not already done)

```bash
# Set your name
git config user.name "Your Name"

# Set your email
git config user.email "your.email@example.com"

# Verify
git config user.name
git config user.email
```

### 4. Add All Files

```bash
# Stage all files
git add .

# Verify what will be committed
git status
```

### 5. Create Initial Commit

```bash
git commit -m "feat: initial release v0.2.0 - Enterprise Salesforce API client

- OAuth 2.0 with automatic token refresh
- Intelligent caching with TTL/TTI
- Retry logic with exponential backoff
- Rate limiting to respect API quotas
- Automatic pagination for large datasets
- Type-safe query builder
- Full CRUD operations
- Comprehensive error handling
- Zero unsafe code
- Production-ready architecture"
```

### 6. Add GitHub Remote

Replace `YOUR_USERNAME` with your actual GitHub username:

```bash
git remote add origin https://github.com/YOUR_USERNAME/salesforce-client.git
```

### 7. Rename Branch to Main

```bash
git branch -M main
```

### 8. Push to GitHub

```bash
git push -u origin main
```

**If push fails with authentication error, see "Authentication Issues" below.**

### 9. Verify on GitHub

1. Go to your repository: `https://github.com/YOUR_USERNAME/salesforce-client`
2. Verify all files are there
3. Check that README.md displays correctly

## Authentication Issues

GitHub no longer accepts password authentication for git operations. You need to use:

### Option A: Personal Access Token (Recommended)

1. Go to: https://github.com/settings/tokens
2. Click "Generate new token" → "Generate new token (classic)"
3. Give it a name: "Salesforce Client Push"
4. Select scopes: `repo` (full control of private repositories)
5. Click "Generate token"
6. **Copy the token immediately** (you won't see it again)
7. When pushing, use the token as your password:
   - Username: your GitHub username
   - Password: the token you just copied

### Option B: SSH Key

1. Generate SSH key:
```bash
ssh-keygen -t ed25519 -C "your.email@example.com"
```

2. Add to GitHub:
   - Go to: https://github.com/settings/keys
   - Click "New SSH key"
   - Paste your public key from `~/.ssh/id_ed25519.pub`

3. Use SSH URL instead:
```bash
git remote set-url origin git@github.com:YOUR_USERNAME/salesforce-client.git
git push -u origin main
```

## After Successful Push

### 1. Configure Repository Settings

On GitHub, go to your repository settings:

**General tab:**
- Add topics: `rust`, `salesforce`, `api-client`, `oauth`, `async`, `tokio`
- Enable Issues

**Actions tab:**
- Enable GitHub Actions (CI will run automatically)

### 2. Create Release Tag (Optional)

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0 - Enterprise Edition"

# Push tag
git push origin v0.2.0
```

Then on GitHub:
1. Go to "Releases" → "Create a new release"
2. Choose tag: v0.2.0
3. Title: "v0.2.0 - Enterprise Edition"
4. Copy description from CHANGELOG.md
5. Click "Publish release"

### 3. Verify CI Pipeline

1. Go to "Actions" tab
2. Watch the CI workflow run
3. Ensure all checks pass

## Troubleshooting

### "Permission denied (publickey)"
- Your SSH key is not set up correctly
- Use Personal Access Token instead (Option A above)

### "remote: Repository not found"
- Check the repository URL is correct
- Verify the repository exists on GitHub
- Ensure you have access to it

### "Updates were rejected because the remote contains work"
- You may have initialized the GitHub repo with files
- Delete and recreate the GitHub repository without any files

### "failed to push some refs"
- Make sure you've committed your changes: `git commit -m "message"`
- Try: `git pull origin main --rebase` then `git push`

## Verification Checklist

After pushing, verify:

- [ ] All files appear on GitHub
- [ ] README.md displays correctly on main page
- [ ] CI workflow runs successfully
- [ ] All directories (src/, examples/, benches/) are present
- [ ] License files are visible

## Quick Reference Commands

```bash
# Check status
git status

# View commit history
git log --oneline

# View remote URL
git remote -v

# Push to GitHub
git push origin main

# Create and push tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

## Getting Help

If you encounter issues:
1. Check GitHub's documentation: https://docs.github.com/en/get-started
2. Verify git configuration: `git config --list`
3. Test connection: `ssh -T git@github.com` (for SSH) or visit GitHub in browser (for HTTPS)

---

**Good luck with your deployment!**
