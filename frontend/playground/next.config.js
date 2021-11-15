module.exports = {
  reactStrictMode: true, // was there by default
  webpack: (config, { buildId, dev, isServer, defaultLoaders, webpack }) => {
    config.module.rules.push({
      test: /\.ints$/i,
      loader: "raw-loader",
    });
    return config;
  },
};
