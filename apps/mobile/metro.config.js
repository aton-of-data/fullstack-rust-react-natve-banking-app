const { getDefaultConfig } = require('expo/metro-config');

/** Metro bundler config — Expo SDK 52+ auto-configures monorepo resolution. */
module.exports = getDefaultConfig(__dirname);
