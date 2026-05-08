# Troubleshooting Guide

## Installation Issues

### Rust Build Failures
**Problem**: `cargo build` fails with linker errors
**Solution**: 
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential libssl-dev pkg-config

# Windows
# Install Visual Studio Build Tools
```

### Node.js Issues
**Problem**: `pnpm install` fails
**Solution**:
```bash
# Clear cache
pnpm store prune

# Reinstall
rm -rf node_modules
pnpm install
```

## Runtime Issues

### Database Locked
**Problem**: "database is locked" error
**Cause**: Multiple processes accessing SQLite
**Solution**:
- Close other Truffle instances
- Check for zombie processes
- Restart application

### AI Model Not Loading
**Problem**: Entity extraction fails
**Cause**: Missing or corrupted model files
**Solution**:
```bash
# Clear model cache
rm -rf ~/.truffle/models/

# Restart app - models will re-download
```

### Sync Conflicts
**Problem**: Sync failures or data conflicts
**Solution**:
1. Pause sync
2. Export data as backup
3. Clear sync state
4. Re-enable sync

## Performance Issues

### Slow Graph Rendering
**Problem**: Graph visualization lags
**Cause**: Too many nodes (>500)
**Solution**:
- Reduce graph depth
- Apply filters
- Use search instead of full graph

### High Memory Usage
**Problem**: App uses >500MB RAM
**Solutions**:
- Restart application
- Clear search index cache
- Reduce entity limit

## Development Issues

### Hot Reload Not Working
**Solution**:
```bash
# Restart Vite
pkill -f vite
pnpm dev
```

### Tests Failing
**Problem**: Tests pass locally but fail in CI
**Common Causes**:
- Platform-specific paths
- Race conditions
- Missing environment variables
**Solution**:
- Use cross-platform path handling
- Add retry logic
- Check CI environment setup

## Error Messages

### "Entity not found"
**Cause**: Entity ID doesn't exist
**Solution**: Check ID format (UUID)

### "Contradiction detected"
**Cause**: Conflicting facts in knowledge graph
**Solution**: Review validation dashboard

### "Merge failed"
**Cause**: Entities have incompatible data
**Solution**: Manually resolve conflicts

## Getting Help

### Logs Location
- **macOS**: `~/Library/Logs/truffle/`
- **Windows**: `%APPDATA%\truffle\logs\`
- **Linux**: `~/.local/share/truffle/logs/`

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug pnpm tauri dev
```

### Support Channels
- GitHub Issues
- Discord
- Email: support@truffle.dev
