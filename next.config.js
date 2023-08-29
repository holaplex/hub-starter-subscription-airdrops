/** @type {import('next').NextConfig} */
const withGraphql = require("next-plugin-graphql");

const nextConfig = {
  async redirects() {
    return [
      {
        source: "/",
        destination: "/airdrops",
        permanent: true,
      },
    ];
  },
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "**.googleusercontent.com",
        pathname: "**",
      },
    ],
  },
  experimental: {
    appDir: true,
  },
  webpack(config) {
    config.experiments = { ...config.experiments, topLevelAwait: true };
    return config;
  },
};

module.exports = withGraphql(nextConfig);
