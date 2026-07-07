# Copy to repo root or frontend/ when scaffolding

npm install --save-dev husky @commitlint/cli @commitlint/config-conventional

npx husky init

# Add commit-msg hook:
echo 'npx --no -- commitlint --edit $1' > .husky/commit-msg
chmod +x .husky/commit-msg

# Copy commitlint.config.cjs from repo root (already present)
