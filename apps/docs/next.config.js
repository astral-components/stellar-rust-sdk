/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ["@stellar-primitives/sdk"],
};

module.exports = nextConfig;
