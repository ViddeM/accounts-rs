/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: true,

  async rewrites() {
    if (process.env.NODE_ENV === "development") {
      return [
        {
          source: "/api/:ep*",
          destination: `http://host.docker.internal:8000/api/:ep*`,
        },
        {
          source: "/.well-known/:ep*",
          destination: "http://host.docker.internal:8000/.well-known/:ep*",
        },
      ];
    } else {
      return [];
    }
  },
};
